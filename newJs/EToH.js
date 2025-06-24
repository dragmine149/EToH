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
@typedef {import('./constants')}
*/


/**
* Returns the word that describes the number.
* @param {number} difficulty The difficuty of the tower.
* @returns The word to describe it.
*/
function getDifficultyWord(difficulty) { return DIFFICULTIES[Math.trunc(difficulty) - 1] ?? "Unknown_Difficulty"; }

/**
* Translates the number form into a more readable word form. Defaults to "Baseline Unknown" if it can't find anything.
* @param {number} difficulty The difficulty of the tower
* @returns {string} The word form of the difficulty
*/
function getDifficulty(difficulty) {
  let stage = Math.trunc(difficulty);
  let sub = difficulty % 1;

  let stageWord = DIFFICULTIES[stage - 1] || "Unknown_Difficulty";
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

/**
* Returns the amount of points that the type provided is worth.
* @param {TOWER_TYPE} type The type of tower.
*/
function getTowerPoints(type) {
  switch (type) {
    case TOWER_TYPE.Obelisk: return 3;
    case TOWER_TYPE.Citadel: return 2;
    case TOWER_TYPE.Tower: return 1;
    case TOWER_TYPE.Steeple: return 0.5;

    default:
    case TOWER_TYPE.Mini_Tower:
    case TOWER_TYPE.Other: return 0;
  }
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

  /** @param {boolean} hover */
  get_name_field(hover) {
    return hover ? this.name : this.shortName;
  }


  /** @param {boolean} hover */
  get_information_field(hover) { return hover ? `${getDifficulty(this.difficulty)} (${this.difficulty})` : getDifficultyWord(this.difficulty); }

  search(search_data) { search_data[this.shortName] = this.name; super.search(search_data); }

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

  /**
  * Returns the shortened version of the text, in accordance to tower format.
  * @param {string} text The text to shorten.
  */
  #short(text) {
    // Tower codes are made up of:
    // each word
    return text.split(/[\s-]/gm)
      // lowered
      .map(word => word.toLowerCase())
      // for 'of' and 'and' to be lower, and the rest upper.
      .map(word => (word == 'of' || word == 'and') ? word[0] : word[0].toUpperCase())
      // and combined.
      .join('');

  }

  get shortName() { return this.#short(this.name); }
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
  /** @type {{badgeId: number, date: number, userId: number}[]} The users completed badges */
  completed = [];

  #all_loaded;
  /** @type {number} When the user can next load all badges, without developer access. */
  set all_loaded(v) {
    this.verbose.log(`All loaded: ${v}`);
    this.#all_loaded = v;
    userManager.storeUser(this.database);
  }
  get all_loaded() { return this.#all_loaded; }

  constructor(user_data) {
    super(user_data);
    this.verbose.log("test");
    if (typeof user_data === 'object' && user_data !== null) this.all_loaded = user_data.all;
  }

  get database() {
    return {
      id: this.id,
      name: this.name,
      display: this.display,
      past: this.past,
      last: this.last,
      all: this.all_loaded
    }
  }

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
    super.postCreate();

    document.getElementById("update").querySelector("[tag='all']").disabled = new Date().getTime() < this.all_loaded;
    document.getElementById("update").querySelector("[tag='all']").title = new Date().getTime() < this.all_loaded ? "Updated within the last week, please wait." : "";

    this.verbose.info("Loading completed badges");

    this.verbose.info(`Loading badges from storage`);
    this.completed = await etohDB.badges.where({ userId: this.id }).toArray();
    etohUI.loadUser(this);

    this.verbose.info("Checking to see if any uncompleted badge has been completed");
    await this.loadUncompleted();

    this.verbose.info("Post Create has been completed!");
  }

  async loadUncompleted() {
    etohUI.reset_new();
    this.verbose.info("Attempting to update uncompleted badges");
    await this.loadBadges(badgeManager.uncompleted(this.completed.map(badge => badge.badgeId)),
      (json) => {
        this.verbose.info(`Found new uncompleted badge: ${json.badgeId} (${badgeManager.ids(json.badgeId)[0].name})`);
        etohUI.loadBadge(json.badgeId, json.date, true);
      });
    this.verbose.info("Uncompleted badges updated!");
  }
  async loadAll() {
    if (new Date().getTime() < this.all_loaded) {
      this.verbose.info("All badges loaded recently... Failed to load");
      return;
    }
    this.verbose.info("Attempting to load all badges");
    // LOCAL (Yes local) check to see when was the last time we did this.
    // Yes, they could techniaclly bypass this / delete data / etc. But those users are "power" users who know how to use the API anyway.
    let date = new Date().getTime();
    date += 604800000;
    this.all_loaded = date;

    etohUI.reset_new();
    await this.loadBadges(badgeManager.ids(), (json) => {
      etohUI.loadBadge(json.badgeId, json.date);
    }, false)
    this.verbose.info("Completed loading all badges!");
  }

  /**
  * Load request badges from the server and stores them.
  * @param {number[]} badges The badges to load.
  * @param {(badge: {badgeId: number, date: number}) => void} callback What to do upon receiving a badge. (other than storing it)
  * @param {boolean} local_check To check if we already have that badge stored before updating it. Defaults to true.
  */
  async loadBadges(badges, callback, local_check) {
    if (!local_check) {
      this.completed = [];
    }

    local_check = local_check !== undefined ? local_check : true;
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
      /** @type {{badgeId: number, date: number}} */
      let info = JSON.parse(line);
      // skip "loading" it if we already have it.
      if (local_check && this.completed.map(b => b.badgeId).includes(info.badgeId)) {
        this.verbose.warn(`Received ${line} from server even though we already have that badgeId...`);
        return;
      }

      this.completed.push(info);
      if (callback) callback(info);
    });

    this.verbose.info(`Storing badges`);
    this.storeCompleted();
    this.verbose.info(`Badges loaded and stored`);
  }

  async storeCompleted() {
    let completed = this.completed.map((b) => {
      return {
        userId: this.id,
        ...b
      }
    });
    // this.verbose.log(completed);
    // await etohDB.badges.bulkPut(completed, undefined, { allKeys: true })
    await etohDB.badges.bulkPut(completed)
      .then((v) => { this.verbose.info(`Finished storing all badges in database`); this.verbose.log(v) })
      .catch(e => {
        this.verbose.error(`Failed to add in ${completed.length - e.failures.length} badges`, e);
      });
  }
}

class EToHUI extends UI {
  show() { super.show(); if (this.main_search) this.main_search.hidden = true; }
  hide() { super.hide(); if (this.main_search) this.main_search.hidden = false; }

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

    this.main_search = document.getElementById("search");

    // Update the elements to show better data.
    badgeManager.name().forEach(/** @param {String} badge_name */(badge_name) => {
      /** @type {Badge[]} Converting from name to object. */
      let badge = badgeManager.name(badge_name)[0];

      // Add the badge to the types total list. Ignoring those towers which are "temporary" as the badge is limited edition.
      if (badge instanceof Tower && badge.category != "temporary") this.types[badge.type].total += 1;
      if (badge instanceof Other) this.types.Other.total += 1;

      // Update the classes
      // this.verbose.log(badge);
      if (badge instanceof Tower) this.set_classes(badge_name, [], ["difficulty", getDifficultyWord(badge.difficulty).toLowerCase()]);
    });
    this.updateTowerCountUI();
    this.syncSize();

    let points = document.getElementById("points");
    let counts = document.getElementById("count");
    points.onmouseover = () => counts.hidden = false;
    points.onmouseleave = () => counts.hidden = true;

    let difficultyCategory = this.setCategory("difficulty");
    badgeManager
      .difficulty()
      .sort((a, b) => a > b)
      .forEach((diff) => difficultyCategory.addBadges(
        badgeManager
          .difficulty(diff)
          .sort((a, b) => a.difficulty > b.difficulty)
          .filter((b) => b.category == "permanent")
          .flatMap((b) => b.name),
        getDifficultyWord(diff)
      ));

    let otherCategory = this.setCategory("other");
    badgeManager.type(Other).forEach(/** @param {Other} badge */(badge) => otherCategory.addBadges(badge.name, badge.category));
  }

  unload_loaded() {
    this.types.Citadel.achieved = [];
    this.types.Mini_Tower.achieved = [];
    this.types.Obelisk.achieved = [];
    this.types.Other.achieved = [];
    this.types.Steeple.achieved = [];
    this.types.Tower.achieved = [];
    this.updateTowerCountUI();
    document.getElementsByTagName("user")[0].innerText = "No-one!";
    super.unload_loaded();
  }

  /**
  * Load a user onto the UI.
  * @param {EToHUser} user The user to display.
  */
  loadUser(user) {
    this.unload_loaded();
    this.reset_new();
    this.show();
    document.getElementsByTagName("user")[0].innerText = user.ui_name;
    user.completed
      // bit of a harsh fix, but it works.
      // TODO: Make new class UserBadge where we can store stuff like this?
      .sort((a, b) => a.date < b.date)
      .forEach((completed) => this.loadBadge(completed.badgeId, completed.date));
  }

  /**
  * Load a badge onto the UI.
  * @param {number} badge_id The id of the badge.
  * @param {number} completion The date/time of completion.
  * @param {bool} new_since How the badge been claimed since we last loaded the data.
  */
  loadBadge(badge_id, completion, new_since) {
    /** @type {Badge} */
    let badge = badgeManager.ids(badge_id)[0];
    /** @type {string[]} */
    let type = this.types[badge instanceof Tower ? badge.type : TOWER_TYPE.Other].achieved;
    if (!type.includes(badge.name) && badge.category != "temporary") type.push(badge.name);
    this.update_badge(badge.name, completion, new_since);
    this.updateTowerCountUI();
  }

  updateTowerCountUI() {
    let points = document.getElementById("points");
    let count = document.getElementById("count");
    /**
    * @param {HTMLDivElement} elm
    * @param {TOWER_TYPE} goal
    */
    function update(elm, goal) {
      let achieved = this.types[goal].achieved.length ?? 0;
      let total = this.types[goal].total;
      let percent = achieved / total;
      percent = Number.isNaN(percent) ? 0 : percent * 100;
      elm.innerText = `${goal}: ${achieved}/${total} (${percent.toFixed(2)}%)`;
    }

    update.bind(this)(count.querySelector("[count='NAT']"), TOWER_TYPE.Other);
    update.bind(this)(count.querySelector("[count='Mini']"), TOWER_TYPE.Mini_Tower);
    update.bind(this)(count.querySelector("[count='Steeple']"), TOWER_TYPE.Steeple);
    update.bind(this)(count.querySelector("[count='Tower']"), TOWER_TYPE.Tower);
    update.bind(this)(count.querySelector("[count='Citadel']"), TOWER_TYPE.Citadel);
    update.bind(this)(count.querySelector("[count='Obelisk']"), TOWER_TYPE.Obelisk);

    let tower_points =
      (this.types.Obelisk.achieved.length * 3) +
      (this.types.Citadel.achieved.length * 2) +
      (this.types.Tower.achieved.length * 1) +
      (this.types.Steeple.achieved.length * 0.5);

    let towers_total = 0;
    let towers_completed = 0;
    badgeManager.name().forEach((badge_name) => {
      /** @type {Badge} */
      let badge = badgeManager.name(badge_name)[0];
      if (!(badge instanceof Tower)) return;
      if (badge.category != "permanent" && badge.category != "other") return;
      if (badge.type == TOWER_TYPE.Mini_Tower) return;
      towers_total += 1;
      towers_completed += this.types[badge.type].achieved.includes(badge_name) ? 1 : 0;
    }, 0)

    let towers_percent = (towers_completed / towers_total) * 100;

    points.querySelector("[count='towers']").innerText = `Towers: ${towers_completed}/${towers_total} (${towers_percent.toFixed(2)}%)`;
    points.querySelector("[count='points']").innerText = `Tower Points: ${tower_points} `;
  }

  hiddenSearch = {
    locked: {},
    mini: {}
  }

  hideLocked() {
    let tower_points =
      (this.types.Obelisk.achieved.length * 3) +
      (this.types.Citadel.achieved.length * 2) +
      (this.types.Tower.achieved.length * 1) +
      (this.types.Steeple.achieved.length * 0.5);

    /** @type {EToHUser} */
    let user = userManager.current_user;
    let diffs = {}
    let accounted = [];
    user.completed
      .forEach((v) => {
        /** @type {Badge} */
        // prevent repeats and non towers.
        let badge = badgeManager.ids(v.badgeId)[0];
        if (!(badge instanceof Tower)) return;
        if (accounted.includes(badge.name)) return;
        accounted.push(badge.name);
        // loop through all the difficulties as 1 tower in a higher difficulty is also 1 tower in every difficuty below that.
        for (let i = badge.difficulty; i > 0; i--) {
          let word = getDifficultyWord(i).toLowerCase();
          diffs[word] = (diffs[word] || 0) + getTowerPoints(badge.type);
        }

        // getDifficultyWord(badgeManager.ids(v.badgeId)[0]?.difficulty).toLowerCase()
      })

    areaManager.name().forEach((area) => {
      /** @type {Area} */
      let data = areaManager.name(area)[0];
      let point_required = tower_points >= data.requirements.points;
      let diff_required = Object.entries(data.requirements.difficulties)
        .every(([key, value]) => diffs[key] >= value);

      let locked = !(point_required && diff_required);
      let areaCate = this.categories.get(area);
      if (locked) areaCate.classList.add("locked");
      badgeManager.area(area).forEach((badge) => {
        let badgeNode = this.badges.get(badge.name);
        if (locked) badgeNode.classList.add("locked");

        Object.entries(this.search_data).filter((v) => v[1] == badge.name).forEach((v) => {
          this.hiddenSearch.locked[v[0]] = v[1];
          delete this.search_data[v[0]];
        });
      });
    })
  }

  showLocked() {
    document.querySelectorAll(".locked").forEach((node) => node.classList.remove("locked"));
    Object.entries(this.hiddenSearch.locked).forEach((v) => {
      this.search_data[v[0]] = v[1]
      delete this.hiddenSearch.locked[v[0]];
    });
  }

  hideMini() {
    badgeManager.type(Tower)
      .filter(/** @param {Tower} badge */(badge) => badge.type == TOWER_TYPE.Mini_Tower)
      .forEach(/** @param {Tower} badge */(badge) => {
        let node = this.badges.get(badge.name);
        node.classList.add("mini-hidden");

        Object.entries(this.search_data).filter((v) => v[1] == badge.name).forEach((v) => {
          this.hiddenSearch.mini[v[0]] = v[1];
          delete this.search_data[v[0]];
        });
      });
  }

  showMini() {
    document.querySelectorAll(".mini-hidden").forEach((node) => node.classList.remove("mini-hidden"));
    Object.entries(this.hiddenSearch.mini).forEach((v) => {
      this.search_data[v[0]] = v[1]
      delete this.hiddenSearch.mini[v[0]];
    });
  }

  locked = false;
  toggleLocked(value) {
    if (value === undefined) value = !this.locked;
    value ? this.hideLocked() : this.showLocked();
    document.getElementById("toggle-locked").innerText = `${value ? 'Show' : 'Hide'} Locked Towers`;
    this.locked = value;
    this.load_category(this.current_category);
  }

  mini = false;
  toggleMini(value) {
    if (value === undefined) value = !this.mini;
    value ? this.hideMini() : this.showMini();
    document.getElementById("toggle-mini").innerText = `${value ? 'Show' : 'Hide'} Mini Towers`;
    this.mini = value;
    this.load_category(this.current_category);
  }

  onCategoryLoad(category_name) {
    this.categories.get(category_name).querySelectorAll("[tag='badge']").forEach((node) => {
      node.hidden = node.locked;
    });
    super.onCategoryLoad(category_name);
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
    ui.showError(`Failed to parse other_data.json: ${data.error} `, true);
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
    ui.showError(`Failed to parse other_data.json: ${data.error} `, true);
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

    checkbox.checked = localStorage.getItem(`setting - Debug - ${checkbox.id} `) === 'true';
  });
})
