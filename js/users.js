class UserManager {
  /** @type {{id: number, name: string, played: boolean}} Roblox user information */
  user;

  /** @type {{
  *   rings: Object.<string, Object.<string, Date>>,
  *   zones: Object.<string, Object.<string, Date>>
  * }} */
  tower_data;

  /**
   * Process tower data and fill badge IDs array
   * @param {Object.<string, Object.<string, TowerData>>} towerData Tower data to process
   */
  getBadgeIds(towerData) {
    let badgeIds = [];
    for (const area of Object.values(towerData)) {
      for (const tower of Object.values(area)) {
        badgeIds.push(tower.badge_id);
        if (tower.old_id) badgeIds.push(tower.old_id);
      }
    }
    return badgeIds;
  }

  /**
  * Get the user id/name from any of the following formats:
  * - https://www.roblox.com/users/605215929/profile
  * - dragmine149
  * - 605215929
  * @param {(string | number)} user The user name/id to get the id/name of.
  * @returns {Promise<{id: number, name: string, played: boolean}>} A dictionary of the user id + name
  */
  async __getUserData(user) {
    async function get(info, name = false) {
      // Test for id
      let response = await fetch(`${CLOUD_URL}/users/${info}${name ? "/name" : "/id"}`);

      if (!response.ok) {
        showError(`Failed to fetch user ${name ? "name" : "id"} for ${info}. (status: ${response.status} ${response.statusText})`, true);
        return null;
      }

      let data = await response.json();
      return name ? data.name : data.id;
    }

    let data = {};

    // attempt to see if input is JUST id.
    if (/^[0-9]*$/.test(user)) {
      data.id = parseInt(user);
    }

    // attempt to see if input is URL
    if (user.includes('roblox.com/users')) {
      let id = user.split('/users/')[1].split('/')[0];
      data.id = parseInt(id);
    }

    // Set the name to the user if we have no id
    if (!data.id) {
      data.name = user;
    }

    // query the database to see if we already have the user
    this.verbose.log(`Attempting to get: `, data, `from storage`);
    let dbuser = await towersDB.users.get(data);
    if (dbuser != undefined) {
      // return if we do
      return dbuser;
    }

    // query the server if we do not
    if (!data.id) {
      data.id = await get(data.name, false);
    }
    if (!data.name) {
      data.name = await get(data.id, true);
    }

    this.verbose.log(`Storing: `, data);

    await towersDB.users.put(data);
    return data;
  }

  async checkPlayed() {
    this.verbose.log("Checking if user has played EToH");
    updateLoadingStatus("Checking if user has played EToH");
    let data = await towersDB.users.get(this.user.id);
    if (data == undefined) {
      data = this.user;
    }

    if (data.played) {
      updateLoadingStatus("User has played EToH (retrieved from storage). Loading user...");
      return data;
    }

    data.played = await badgeManager.hasBadge(data.id, "First Tower") > 0;
    await towersDB.users.put(data);
    updateLoadingStatus(data.played ? "User has played EToH (retrieved from server). Loading user..." : "User has not played EToH (retrieved from server).");
    return data;
  }

  async loadUserData() {
    // Assumption: User has already been checked for already loaded.
    if (!this.user.played) {
      return;
    }
    // attempt loading from storage.
    let towers = await towersDB.towers.where({ user_id: this.user.id });
    this.verbose.log(towers);
    if (towers != undefined) {
      updateLoadingStatus("User has tower data, loading from storage");
      return;
    }

    this.verbose.log("Loading data from server as not in storage.");
    let ids = Object.keys(towerManager.tower_ids);
    let request = new Request(`${CLOUD_URL}/towers/${this.user.id}/all`, {
      method: 'POST',
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        "badgeids": ids
      })
    });

    await network.requestStream(request, async (line) => {
      // then insert the data (upon conversion) into the database.
      let badge = await noSyncTryCatch(() => JSON.parse(line));
      if (badge.error) {
        showError(`Failed to parse badge data: ${badge.error}. Please try again later. (roblox api might be down)`);
      }

      /** @type {{badgeId: number, date: number}} */
      let badgeData = badge.data;
      // console.log(badgeData);

      towersDB.towers.put({
        badge_id: badgeData.badgeId,
        user_id: this.user.id,
        completion: badgeData.date
      });

      updateLoadingStatus(`Loaded: ${towerManager.tower_ids[badgeData.badgeId]} from server`)
      // console.log(badgeData);
    })
  }

  /**
  * Create a new user class.
  * @param {string} user
  */
  constructor(user) {
    updateLoadingStatus(`Attempting to load: ${user}`, true);

    this.verbose = new Verbose(`UserManager`, '#6189af');

    (async () => {
      this.user = await this.__getUserData(user);
      this.user = await this.checkPlayed();

      if (!this.user.played) {
        let badge = badgeManager.badge(2125419210);
        updateLoadingStatus(`Cancelling loading of ${this.user.name} as they have not yet played EToH.<br>
User must have '<a href=${badge.link} target="_blank" rel="noopener noreferrer">Beat your first tower</a>' badge before they can be viewed here.`);
        userData.clearUser();
        return;
      }

      await this.loadUserData();
    })();
  }

  storeUser() {
    return {
      id: this.user,
      towers: this.tower_data
    }
  }
}

class UserData {
  /** @type {UserManager[]} */
  users;

  /** @type {UserManager} */
  currentUser;

  constructor() {
    this.users = [];
    this.currentUser = null;
  }

  /**
  * Search a user and loads their information
  */
  searchUser() {
    hideError();

    let searchElm = document.getElementById("search_input").value;
    this.currentUser = new UserManager(searchElm);
    this.users.push(this.currentUser);
  }

  clearUser() {
    this.currentUser = null;
  }
}

let userData = new UserData();
