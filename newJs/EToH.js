/*global tryCatch, badgeManager, Badge, User, network, UserManager, ui, etohDB, logs, TowerManager, areaManager, Area, CLOUD_URL */
/*eslint no-undef: "error"*/
/*exported Tower, Other, EToHUser, userManager, towerManager, miniSearch, endMiniSearch, TOWER_TYPE, DIFFICULTIES, SUB_LEVELS, pointsFromType */


/**
@typedef {{
  name: string,
  difficulty: number,
  badges: number[]
}} ServerTower

@typedef {{
  easy: number?,
  medium: number?,
  hard: number?,
  difficult: number?,
  challenging: number?,
  intense: number?,
  remorseless: number?,
  insane: number?,
  extreme: number?,
  terrifying: number?,
  catastrophic: number?
}} ServerDifficulties

@typedef {{
  name: String,
  requirements: {
    difficulties: ServerDifficulties,
    points: number
  },
  sub_area: string?,
  towers: ServerTower[]
}} ServerAreas

@typedef {{
  areas: {
    (type: string): ServerAreas[]
  }
}} ServerTowers

@typedef {{
  name: String,
  category: String,
  badges: number[]
}} ServerOther

@typedef {{
  data: ServerOther[]
}} ServerOtherParent

*/

class Tower extends Badge {
  /**
  * Makes a new tower badge.
  * @param {String} name FULL NAME of the tower.
  * @param {number[]} ids List of tower badge ids.
  * @param {number} difficulty The difficulty of the tower
  * @param {String} area The area where the tower is located
  */
  constructor(name, ids, difficulty, area) {
    super(name, ids);
    this.__addProperty('difficulty', difficulty);
    this.__addProperty('area', area);
  }

  get shortName() {
    // Tower codes are made up of:
    // each word
    return this.name.split(' ')
      // lowered
      .map(word => word.toLowerCase())
      // for 'of' and 'and' to be lower, and the rest upper.
      .map(word => (word == 'of' || word == 'and') ? word[0] : word[0].toUpperCase())
      // and combined.
      .join('');
  }
}

class Other extends Badge {
  /**
  * Makes a new "Other" type badge.
  * @param {string} name Name of the badge.
  * @param {number[]} ids Ids associated with the badge.
  * @param {String} category The category the badge belongs in.
  */
  constructor(name, ids, category) {
    super(name, ids);
    this.__addProperty('category', category);
  }
}

// EToHUser, extension of User designed for specifically targeting EToH
// TODO: Split part of this code up into a new class "BadgeUser"?
class EToHUser extends User {
  /** @type {{badgeId: number, date: number}[]} The users completed badges */
  completed = [];

  static async create(user_data, db) {
    // have to call the parent function.
    let result = await User.create(user_data, db);
    console.log(result);
    if (!Number.isNaN(Number(result)) && result !== true) {
      console.log('number user');
      return result;
    }
    console.log(result instanceof User);
    if (result !== true && !(result instanceof User)) {
      console.log(`Some sort of error...`);
      return null;
    }

    // If we have loaded the user recently, then we must have played.
    if (result.last >= 0) {
      return new EToHUser(result.database);
    }

    // request the server to see if we have played or not.
    result.verbose.info(`Checking if user has played`);

    /** @type {number[]} */
    let played = badgeManager.names("Played")[0].ids;
    result.verbose.debug(played);
    let hasPlayed = await network.getEarlierBadge(result.id, played[0], played[1]);
    if (hasPlayed.earliest > 0) {
      result.verbose.debug(`Upgrading user to type ETOH`);
      return new EToHUser(result.database);
    }
    return null;
  }

  /**
  * Function is called upon the user finish loading from userManager.findUser()
  */
  async postCreate() {
    this.verbose.info("Loading completed badges");

    this.verbose.info(`Loading badges from storage`);
    this.completed = await etohDB.badges.toArray();
    towerManager.loadUI(this);

    this.verbose.info("Checking to see if any uncompleted badge has been completed");
    await this.loadBadges(badgeManager.uncompleted(this.completed
      .map(badge => badge.badgeId)
    ),
      /** @param {{badgeId: number, date: number}} json */
      (json) => {
        towerManager.loadUI(json.badgeId, true);
      }
    );

    this.verbose.info("Post Create has been completed!");
  }

  async loadUncompleted() {
    this.verbose.info("Attempting to update uncompleted badges");
    await this.loadBadges(badgeManager.uncompleted(this.completed
      .map(badge => badge.badgeId)
    ),
      /** @param {{badgeId: number, date: number}} json */
      (json) => {
        this.verbose.info(`Found new uncompleted badge: ${json.badgeId}`);
        towerManager.loadBadge(json.badgeId, true);
      });
    this.verbose.info("Uncompleted badges updated!");
  }

  async loadBadges(badges, callback) {
    this.verbose.info(`Loading badges from server`);
    await network.requestStream(new Request(`${CLOUD_URL}/badges/${this.id}/all`, {
      method: 'POST',
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        'badgeids': badges
      })
    }), (line) => {
      this.completed.push(JSON.parse(line));
      if (callback) callback(JSON.parse(line));
    });

    this.verbose.info(`Storing badges`);
    this.storeCompleted();
    this.verbose.info(`Badges loaded and stored`);
  }

  storeCompleted() {
    let completed = this.completed.map((b) => {
      return {
        userId: this.id,
        ...b
      }
    });
    etohDB.badges.bulkPut(completed);
  }
}

async function loadTowersFromServer() {
  let server_tower = await fetch('data/tower_data.json');
  if (!server_tower.ok) {
    ui.showError(`Failed to fetch tower_data.json: ${server_tower.status} ${server_tower.statusText}.`, true);
    return;
  }

  /** @type {{data: ServerTowers | null, error: Error | null}} */
  let data = await tryCatch(server_tower.json());

  if (data.error) {
    ui.showError(`Failed to parse other_data.json: ${data.error}`, true);
    return;
  }
  Object.entries(data.data.areas).forEach(
    /**
    * @param {[String, ServerAreas[]]} areas
    */
    (areas) => {
      areas[1].forEach((area) => {
        area.towers.forEach((tower) => {
          badgeManager.addBadge(new Tower(tower.name, tower.badges, tower.difficulty, area.name));
        });
        areaManager.addArea(new Area(area.name, area.sub_area, area.requirements));
      })
    })
}

async function loadOthersFromServer() {
  let server_other = await fetch('data/other_data.json');
  if (!server_other.ok) {
    ui.showError(`Failed to fetch tower_data.json: ${server_other.status} ${server_other.statusText}.`, true);
    return;
  }

  /** @type {{data: ServerOtherParent | null, error: Error | null}} */
  let data = await tryCatch(server_other.json());

  if (data.error) {
    ui.showError(`Failed to parse other_data.json: ${data.error}`, true);
    return;
  }

  data.data.data.forEach((badge) => {
    badgeManager.addBadge(new Other(badge.name, badge.badges, badge.category));
  })
}


function miniSearch() {
  document.getElementsByTagName('user')[0].hidden = true;

  let miniSearch = document.getElementById('mini-search');

  miniSearch.hidden = false;
  if (miniSearch.value === "") {
    miniSearch.value = this.currentUser.user.name;
  }

  miniSearch.focus();
}
function endMiniSearch() {
  document.getElementById('mini-search').hidden = true;
  document.getElementsByTagName('user')[0].hidden = false;
}


// Shows the user what we are doing in the background. Always good to keep them up to date.
logs.addCallback("*", logs.serveriety.INFO,
  /** @param {{category: String, serveriety: String, params: any[], trace: String | undefined, time: dayjs}} log */
  (log) => {
    document.querySelector("[tag='status']").innerText = log.params.toString();
  });
// Shows the user any important errors they should be cautious of.
logs.addCallback("*", logs.serveriety.ERROR,
  /** @param {{category: String, serveriety: String, params: any[], trace: String | undefined, time: dayjs}} log */
  (log) => {
    document.getElementById("errors").hidden = false;
    document.getElementById("error_message").innerText = log.params.toString();
  });

badgeManager.addFilter('difficulty', b => Math.floor(b.difficulty));
badgeManager.addFilter('area', b => b.area);
badgeManager.addFilter('category', b => b.category);

let userManager = new UserManager(etohDB);
userManager.limit = 250;
userManager.userClass = EToHUser;
userManager.unload_callback = () => {
  towerManager.unloadUI();
}

async function loadData(callback) {
  await loadTowersFromServer();
  await loadOthersFromServer();
  callback();
}

let towerManager = new TowerManager();
loadData(() => {
  userManager.loadURL();
});

addEventListener('DOMContentLoaded', () => {
  // update the settings checkboxes.
  let checkboxes = document.querySelectorAll('.settings input[type="checkbox"]');
  checkboxes.forEach(checkbox => {
    if (!checkbox.id.startsWith("verbose")) {
      return;
    }

    checkbox.checked = localStorage.getItem(`setting-Debug-${checkbox.id}`) === 'true';
  });
})
