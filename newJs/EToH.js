/*global tryCatch, badgeManager, Badge, User, network, UserManager, ui, etohDB, logs, TOWER_TYPE, DIFFICULTIES, SUB_LEVELS, areaManager, Area, CLOUD_URL, UI */
/*eslint no-undef: "error"*/
/*exported Tower, Other, EToHUser, userManager, towerManager, miniSearch, endMiniSearch, TOWER_TYPE, DIFFICULTIES, SUB_LEVELS, pointsFromType */

/**
@typedef {{
  name: string,
  difficulty: number,
  badges: number[],
  type: string?
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

@typedef {import('./BadgeManager')}
@typedef {import('./user')}
@typedef {import('./network')}
@typedef {import('./main')}
@typedef {import('./AreaManager')}
@typedef {import('./Ui')}
*/


/**
* Returns the word that describes the number.
* @param {number} difficulty The difficuty of the tower.
* @returns The word to describe it.
*/
function getDifficultyWord(difficulty) { return DIFFICULTIES[Math.trunc(difficulty) - 1] ?? "Unknown"; }

/**
* Translates the number form into a more readable word form. Defaults to "Baseline Unknown" if it can't find anything.
* @param {number} difficulty The difficulty of the tower
* @returns {string} The word form of the difficulty
*/
function getDifficulty(difficulty) {
  let stage = Math.trunc(difficulty);
  let sub = difficulty % 1;

  let stageWord = DIFFICULTIES[stage - 1] || "Unknown";
  let subWord = SUB_LEVELS.find(level => sub >= level.threshold)?.name || "Baseline";

  return `${subWord} ${stageWord}`;
}

/**
* Returns what type the tower is from its name. (Please don't make this too confusing EToH Devs...)
* @param {String} name The name of the tower.
* @param {String} type The type to use when we can't determine from name or from badge.
* @returns The type of tower.
*/
function getTowerType(name, type) {
  if (name.startsWith("Steeple")) return TOWER_TYPE.Steeple;
  if (name.startsWith('Tower of') || name == 'Thanos Tower') return TOWER_TYPE.Tower;
  if (name.startsWith('Citadel of')) return TOWER_TYPE.Citadel;
  if (name.startsWith('Obeisk of')) return TOWER_TYPE.Obelisk;
  let badge = badgeManager.name(name)[0];
  if (badge instanceof Tower) return badge.type;
  if (type) return type;
  return TOWER_TYPE.Other;
}

class Tower extends Badge {
  /** @type {number} The difficulty of the tower. */
  difficulty;
  /** @type {string} The area where the tower is located */
  area;
  /** @type {string} The type of the tower. */
  type;
  /** @type {string} What "category" this comes under as, "permanent", "temporary", "other" */
  category;

  /**
  * Makes a new tower badge.
  * @param {String} name FULL NAME of the tower.
  * @param {number[]} ids List of tower badge ids.
  * @param {number} difficulty The difficulty of the tower
  * @param {String} area The area where the tower is located
  * @param {String?} type The type of the tower if it can't be determined by name.
  */
  constructor(name, ids, difficulty, area, type, category) {
    super(name, ids);
    this.__addProperty('difficulty', difficulty);
    this.__addProperty('area', area);
    this.__addProperty('type', getTowerType(name, type));
    this.__addProperty('category', category);
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
  /** @type {string} The category the badge belongs to */
  category;

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
    let played = badgeManager.name("Played")[0].ids;
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
    this.completed = await etohDB.badges.where({ userId: this.id }).toArray();
    etohUI.loadUser(this);

    this.verbose.info("Checking to see if any uncompleted badge has been completed");
    await this.loadUncompleted();

    this.verbose.info("Post Create has been completed!");
  }

  async loadUncompleted() {
    this.verbose.info("Attempting to update uncompleted badges");
    await this.loadBadges(badgeManager.uncompleted(this.completed.map(badge => badge.badgeId)).flatMap(badge => badge.ids),
      (json) => {
        this.verbose.info(`Found new uncompleted badge: ${json.badgeId}`);
        etohUI.loadBadge(json.badgeId, json.date);
      });
    this.verbose.info("Uncompleted badges updated!");
  }

  /**
  * Load request badges from the server and stores them.
  * @param {number[]} badges The badges to load.
  * @param {(badge: {badgeId: number, date: number}) => void} callback What to do upon receiving a badge. (other than storing it)
  */
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

class EToHUI extends UI {
  show() { super.show(); if (this.search) this.search.hidden = true; }
  hide() { super.hide(); if (this.search) this.search.hidden = false; }

  // How many typees of each badge there is.
  types = {
    Mini_Tower: { total: 0, achieved: [] },
    Steeple: { total: 0, achieved: [] },
    Tower: { total: 0, achieved: [] },
    Citadel: { total: 0, achieved: [] },
    Obelisk: { total: 0, achieved: [] },
    Other: { total: 0, achieved: [] }
  }

  constructor() {
    // create a list of categories.
    let categories = [];
    categories = categories.concat(areaManager.name());
    categories = categories.concat(badgeManager.category());
    categories.push("other");

    // then get the base class to generate the ui.
    super(categories, /** @param {String} badge_name */(badge_name) => {
      // callback for determinding where badges go.
      /** @type {Badge} Converting from name to object. */
      let badge = badgeManager.name(badge_name)[0];

      if (badge instanceof Tower) return badge.area;
      if (badge instanceof Other) return badge.category;
      // The, "What do i do with this?" Category.
      return "other";
    }, /** @param {String} category */(category) => {
      // Callback for determinding where categories go.

      /** @type {Area[]} */
      let area = areaManager.name(category);
      if (area.length > 0) return area[0].parent ?? "root";
      /** @type {Other[]} */
      // let other = badgeManager.category(category);
      // if (other.length > 0) return "root";
      // Got to have a comment here... for some reason.
      return "root";
    });

    this.search = document.getElementById("search");

    // Update the elements to show better data.
    badgeManager.name().forEach(/** @param {String} badge_name */(badge_name) => {
      /** @type {Badge[]} Converting from name to object. */
      let badge = badgeManager.name(badge_name)[0];

      if (badge instanceof Tower) this.set_data(badge_name, badge.shortName, getDifficultyWord(badge.difficulty));
      if (badge instanceof Tower) this.set_hover(badge_name, badge.name, `${getDifficulty(badge.difficulty)} (${badge.difficulty})`);

      // Add the badge to the types total list. Ignoring those towers which are "temporary" as the badge is limited edition.
      if (badge instanceof Tower && badge.category != "temporary") this.types[badge.type].total += 1;
      if (badge instanceof Other) this.types.Other.total += 1;
      // if (badge instanceof Other) this.set_data(badge_name, badge.shortName, badge.difficulty);
      // if (badge instanceof Other) this.set_hover(badge_name, badge.shortName, badge.difficulty);

      // Update the classes
      // this.verbose.log(badge);
      if (badge instanceof Tower) this.set_classes(badge_name, [], ["difficulty", getDifficultyWord(badge.difficulty).toLowerCase()]);
    });
    this.updateTowerCountUI();
  }

  unload_loaded() {
    this.types.Citadel.achieved = [];
    this.types.Mini_Tower.achieved = [];
    this.types.Obelisk.achieved = [];
    this.types.Other.achieved = [];
    this.types.Steeple.achieved = [];
    this.types.Tower.achieved = [];
    this.updateTowerCountUI();
    super.unload_loaded();
  }

  /**
  * Load a user onto the UI.
  * @param {EToHUser} user The user to display.
  */
  loadUser(user) {
    this.unload_loaded();
    this.show();
    user.completed.forEach((completed) => this.loadBadge(completed.badgeId, completed.date));
  }

  /**
  * Load a badge onto the UI.
  * @param {number} badge_id The id of the badge.
  * @param {number} completion The date/time of completion.
  */
  loadBadge(badge_id, completion) {
    /** @type {Badge} */
    let badge = badgeManager.ids(badge_id)[0];
    /** @type {string[]} */
    let type = this.types[badge instanceof Tower ? badge.type : TOWER_TYPE.Other].achieved;
    if (!type.includes(badge.name) && badge.category != "temporary") type.push(badge.name);
    this.update_badge(badge.name, completion);
    this.updateTowerCountUI();
  }

  updateTowerCountUI() {
    let count = document.getElementById("count");
    /**
    * @param {HTMLDivElement} elm
    * @param {TOWER_TYPE} goal
    */
    function update(elm, goal) {
      elm.innerText = `${goal}: ${this.types[goal].achieved.length}/${this.types[goal].total} (${(this.types[goal].achieved.length / this.types[goal].total * 100).toFixed(2)}%)`;
    }

    update.bind(this)(count.querySelector("[count='NAT']"), TOWER_TYPE.Other);
    update.bind(this)(count.querySelector("[count='Mini']"), TOWER_TYPE.Mini_Tower);
    update.bind(this)(count.querySelector("[count='Steeple']"), TOWER_TYPE.Steeple);
    update.bind(this)(count.querySelector("[count='Tower']"), TOWER_TYPE.Tower);
    update.bind(this)(count.querySelector("[count='Citadel']"), TOWER_TYPE.Citadel);
    update.bind(this)(count.querySelector("[count='Obelisk']"), TOWER_TYPE.Obelisk);
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
          badgeManager.addBadge(new Tower(tower.name, tower.badges, tower.difficulty, area.name, tower.type, areas[0]));
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
    miniSearch.value = userManager.current_user.name;
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
userManager.load_database();
userManager.unload_callback = () => {
  etohUI.unload_loaded();
}

async function loadData(callback) {
  await loadTowersFromServer();
  await loadOthersFromServer();
  callback();
}

// let towerManager = new TowerManager();
/** @type {EToHUI} */
let etohUI;

loadData(() => {
  etohUI = new EToHUI();

  let user_list = document.getElementById("user_list");
  userManager.names().forEach(user => {
    if (user_list === null) return;
    let option_node = document.createElement("option");
    option_node.innerText = user;
    option_node.value = user;
    user_list.appendChild(option_node);
  });

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
