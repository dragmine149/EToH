/*global tryCatch, badgeManager, Badge, User, network, UserManager, ui, etohDB, logs */
/*eslint no-undef: "error"*/
/*exported Tower, Other, EToHUser, userManager */


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

class EToHUser extends User {
  static async create(user_data, db) {
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

    result.verbose.info(`Checking if user has played`);

    /** @type {number[]} */
    let played = badgeManager.names("Played")[0].ids;
    result.verbose.info(played);
    let hasPlayed = await network.getEarlierBadge(result.id, played[0], played[1]);
    if (hasPlayed.earliest > 0) {
      result.verbose.debug(`Upgrading user to type ETOH`);
      return new EToHUser(result.database);
    }
    return null;
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
      areas[1].forEach((area) => area.towers.forEach((tower) => {
        badgeManager.addBadge(new Tower(tower.name, tower.badges, tower.difficulty, area.name));
      }))
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

badgeManager.addFilter('difficulty', b => Math.floor(b.difficulty));
badgeManager.addFilter('area', b => b.area);
badgeManager.addFilter('category', b => b.category);

let userManager = new UserManager(etohDB);
userManager.userClass = EToHUser;

loadTowersFromServer();
loadOthersFromServer();

logs.addCallback("*", "info", (log) => {
  document.querySelector("[tag='status']").innerText = log.params.toString();
})
