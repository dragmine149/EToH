let cloud_url = 'https://etoh-proxy.dragmine149.workers.dev';

class UserManager {
  /** @type {number} Roblox UserID */
  user;

  /** @type {{
  *   rings: Object.<string, Object.<string, Date>>,
  *   zones: Object.<string, Object.<string, Date>>
  * }} */
  tower_data;

  /** @type {Database} */
  database;

  /**
   * Process tower data and fill badge IDs array
   * @param {Object.<string, Object.<string, TowerData>>} towerData Tower data to process
   */
  processTowerData(towerData) {
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
   * Process badge data and update tower completion dates
   * @param {Object.<string, Object.<string, TowerData>>} towerData Tower data to update
   * @param {Array<{badgeId: number, awardedDate: string}>} badgeResults Badge completion data
   */
  updateTowerDates(towerData, badgeResults) {
    let data = {};
    for (const [areaName, area] of Object.entries(towerData)) {
      for (const [towerName, tower] of Object.entries(area)) {
        const defaultDate = dayjs('0000-00-00');

        if (tower.badge_id) {
          const result = badgeResults.find(b => b.badgeId === tower.badge_id);
          data[areaName] = data[areaName] || {};
          data[areaName][towerName] = result ? dayjs(result.awardedDate).toDate() : defaultDate;
        }

        if (tower.old_id) {
          const oldResult = badgeResults.find(b => b.badgeId === tower.old_id);
          if (oldResult) {
            data[areaName] = data[areaName] || {};
            data[areaName][towerName] = dayjs(oldResult.awardedDate);
          }
        }
      }
    }
    return data;
  }

  async loadUserData() {
    let data = {};

    let badgeIds = [];
    badgeIds = this.processTowerData(towers.rings);
    badgeIds.concat(this.processTowerData(towers.zones));

    const response = await fetch(`${cloud_url}/towers/${this.user}/all`, {
      method: 'POST',
      body: JSON.stringify(badgeIds)
    });
    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Failed to fetch badge data (status: ${response.status} ${response.statusText})\n${errorText}`);
      showNotification(`Failed to fetch badge data. (status: ${response.status} ${response.statusText})`);
      return null;
    }
    const badgeData = await response.json();

    data.rings = this.updateTowerDates(towers.rings, badgeData.data);
    data.zones = this.updateTowerDates(towers.zones, badgeData.data);

    this.tower_data = data;
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

    this.database = new Database('ETOH', 1, this.__upgradeDatabase.bind(this));

    (async () => {
      this.user = await this.__getUserID(user);
      await this.loadUserData();
    })();
  }

  __upgradeDatabase(db, oldVersion, newVersion) {
    console.log(`Upgrading database from version ${oldVersion} to ${newVersion}`);
    // Add your upgrade logic here
    console.error("Erm, help...");
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
    this.currentUser.loadUserData();
  }

  saveUser() {
    this.storage.setStorage(this.currentUser.user, this.currentUser.storeUser());
  }
}

let userData = new UserData();
