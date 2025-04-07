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

          let uid = this.user;
          towersDB.transaction('rw', towersDB.towers, function () {
            towersDB.towers.add({
              badge_id: tower.badge_id,
              user_id: uid,
              completion: data[areaName][towerName]
            })
          })
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
    badgeIds = this.processTowerData(towerManager.raw_data.rings);
    badgeIds.concat(this.processTowerData(towerManager.raw_data.zones));

    console.log(badgeIds);

    const response = await fetch(`${cloud_url}/towers/${this.user}/all`, {
      method: 'POST',
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        "badgeids": badgeIds
      })
    });

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
      const { done, value } = await reader.read();

      if (done) break;

      // Add new chunk to buffer
      buffer += decoder.decode(value, { stream: true });

      // Process complete lines from buffer
      const lines = buffer.split('\n');
      // Keep the last (potentially incomplete) line in buffer
      buffer = lines.pop() || '';

      // Process complete lines
      for (const line of lines) {
        if (line.trim()) {
          try {
            /** @type {{badgeId: number, awardedDate: string}} */
            const badge = JSON.parse(line);
            // processBadgeCallback(badge);
            console.log(badge);

            let uid = this.user;
            towersDB.transaction('rw', towersDB.towers, function () {
              towersDB.towers.add({
                badge_id: badge.badgeId,
                user_id: uid,
                completion: badge.awardedDate
              })
            })

          } catch (e) {
            console.error('Failed to parse badge data:', e);
          }
        }
      }
    }

    // Process any remaining data
    if (buffer.trim()) {
      try {
        const badge = JSON.parse(buffer);
        // processBadgeCallback(badge);
        console.log(badge);
      } catch (e) {
        console.error('Failed to parse final badge data:', e);
      }
    }

    // Object.keys(towerManager.raw_data.rings).reduce()

    // Parse the complete JSON response
    // const badgeData = JSON.parse(buffer);

    // data.rings = this.updateTowerDates(towerManager.raw_data.rings, badgeData.data);
    // data.zones = this.updateTowerDates(towerManager.raw_data.zones, badgeData.data);

    // this.tower_data = data;
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
