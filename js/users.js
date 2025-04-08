let cloud_url = 'https://etoh-proxy.dragmine149.workers.dev';

class UserManager {
  /** @type {number} Roblox UserID */
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

  async loadUserData() {
    // pre load badge ids
    let badgeIds = [];
    badgeIds = this.getBadgeIds(towerManager.raw_data.rings);
    badgeIds.concat(this.getBadgeIds(towerManager.raw_data.zones));

    // get the data from the server.
    const response = await fetch(`${cloud_url}/towers/${this.user}/all`, {
      method: 'POST',
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        "badgeids": badgeIds
      })
    });

    // make sure we can actually process the response.
    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Failed to fetch badge data (status: ${response.status} ${response.statusText})\n${errorText}`);
      showNotification(`Failed to fetch badge data. (status: ${response.status} ${response.statusText})`);
      return null;
    }

    // Set up streaming
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      let { done, value } = await reader.read();

      // Add new chunk to buffer
      buffer += decoder.decode(value, { stream: true });

      // Process complete lines from buffer
      const lines = buffer.split('\n');
      // Keep the last (potentially incomplete) line in buffer
      if (!done) {
        buffer = lines.pop() || '';
      }

      // Process complete lines
      for (const line of lines) {
        // cleans up the line making sure it can be used
        if (!line.trim()) {
          continue;
        }

        // then insert the data (upon conversion) into the database.
        let badge = await noSyncTryCatch(() => JSON.parse(line));
        if (badge.error) {
          // showNotification(`Failed to parse badge data: ${badge.error}. Please try again later. (roblox api might be down)`);
        }

        /** @type {{badgeId: number, date: number}} */
        let badgeData = badge.data;
        // console.log(badgeData);

        towersDB.towers.put({
          badge_id: badgeData.badgeId,
          user_id: this.user,
          completion: badgeData.date
        });
      }

      if (done) break;
    }

    console.log(`Loaded user data!`);
  }

  // https://www.roblox.com/users/605215929/profile
  // dragmine149
  // 605215929
  async __getUserID(user) {
    // Test if number is given
    if (/^[0-9]*$/.test(user)) {
      return parseInt(user);
    }

    // Test if URL is given
    if (user.includes('roblox.com/users/')) {
      let id = user.split('/users/')[1].split('/')[0];
      return parseInt(id);
    }

    // Test for username
    let response = await fetch(`https://etoh-proxy.dragmine149.workers.dev/users/${user}`);

    if (!response.ok) {
      console.error(`Failed to fetch user ID for ${user} (status: ${response.status} ${response.statusText})`);
      showNotification(`Failed to fetch user ID for ${user}.`);
      return null;
    }

    let data = await response.json();
    return data.id;
  }

  /**
  * Create a new user class.
  * @param {string} user
  */
  constructor(user) {
    console.log(`Attempting to load: ${user}`);

    (async () => {
      this.user = await this.__getUserID(user);

      if (this.user == null || this.user == undefined) {
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
    this.storage = new DragStorage(`users`);
    this.currentUser = null;
  }

  /**
  * Search a user and loads their information
  */
  searchUser() {
    let searchElm = document.getElementById("search_input").value;
    this.currentUser = new UserManager(searchElm);
    this.users.push(this.currentUser);

    if (this.storage.hasStorage(this.currentUser.user)) {
      this.currentUser.tower_data = this.storage.getStorage(this.currentUser.user);
      return;
    }
    // this.currentUser.loadUserData();
  }

  saveUser() {
    this.storage.setStorage(this.currentUser.user, this.currentUser.storeUser());
  }
}

let userData = new UserData();
let towersDB = new Dexie("Towers");
towersDB.version(1).stores({
  towers: `[badge_id+user_id]`,
})
