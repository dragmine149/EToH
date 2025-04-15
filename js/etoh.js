/**
 * API Requests
 * Docs: https://create.roblox.com/docs/en-us/cloud/legacy/badges/v1#/

 * Badge data from: https://badges.roblox.com/v1/universes/3264581003/badges
 * Awarded data: https://badges.roblox.com/v1/users/605215929/badges/2125363409/awarded-dates

 * Somehow maybe? Get difficult from wiki or smth
*/


/**
 * Represents a single tower
 * @typedef {{
 *   difficulty: number,
 *   badge_id: number,
 *   old_id?: number
 * }} TowerData
 */

/**
 * Represents a ring containing towers
 * @typedef {Object.<string, TowerData>} Ring
 */

/**
 * Represents a zone containing towers
 * @typedef {Object.<string, TowerData>} Zone
 */

/**
 * @typedef {{
 *   rings: Object.<string, Ring>,
 *   zones: Object.<string, Zone>
 * }} Towers
 */

class Tower {
  /** @type {string} */
  name;
  /** @type {string} */
  area;
  /** @type {number} */
  difficulty;
  /** @type {number} */
  badge;
  /** @type {number} */
  old_badge;

  get shortName() {
    return this.name.split(' ').map(word => (word.toLowerCase() == 'of' || word.toLowerCase() == 'and') ? word[0] : word[0].toUpperCase()).join('');
  }
  get difficultyWord() {
    return towerManager.getDifficulty(this.difficulty);
  }

  constructor(name, area, difficulty, badge, old_badge) {
    this.name = name;
    this.area = area;
    this.difficulty = difficulty;
    this.badge = badge;
    this.old_badge = old_badge;
  }
}

class TowerManager {
  /** @type {Tower[]} */
  towers = [];
  areas = [];

  difficulties = ["Easy", "Medium", "Hard", "Difficult", "Challenging", "Intense", "Remorseless", "Insane", "Extreme", "Terrifying", "Catastrophic"];
  subLevels = [
    { threshold: 0.89, name: "Peak" },
    { threshold: 0.78, name: "High-Peak" },
    { threshold: 0.67, name: "High" },
    { threshold: 0.56, name: "Mid-High" },
    { threshold: 0.45, name: "Mid" },
    { threshold: 0.33, name: "Low-Mid" },
    { threshold: 0.22, name: "Low" },
    { threshold: 0.11, name: "Bottom-Low" }
  ];

  elements = {};

  /** @type {Towers} The raw tower data from the server */
  raw_data;
  /** @type {Object.<number, string>} The badge id and the name of said badge. */
  tower_ids = {};

  get __areaElm() {
    return document.getElementById('towers');
  }

  constructor() {
    (async () => {
      await this.loadTowers();
      this.__create_elements();
    })()
  }

  /**
  * Translates the number form into a more readable word form
  * @param {number} difficulty The difficulty of the tower
  * @returns {string} The word form of the difficulty
  */
  getDifficulty(difficulty) {
    let stage = Math.trunc(difficulty);
    let sub = difficulty % 1;

    let stageWord = this.difficulties[stage - 1] || "Unknown";
    let subWord = subLevels.find(level => sub >= level.threshold)?.name || "Bottom";

    return `${stageWord} ${subWord}`;
  }

  __create_elements() {
    this.areas.forEach(areaName => {
      let areaElement = document.createElement('div');
      areaElement.id = `area-${areaName}`;
      areaElement.classList.add('area');
      this.__areaElm.appendChild(areaElement);
      this.elements[areaName] = areaElement;
    });

    this.difficulties.forEach(difficulty => {
      let difficultyElement = document.createElement('div');
      difficultyElement.id = `difficulty-${difficulty}`;
      difficultyElement.classList.add('difficulty');
      this.__areaElm.appendChild(difficultyElement);
      this.elements[difficulty] = difficultyElement;
    });

    this.towers.forEach(tower => {
      let towerElm = document.createElement("div");
      let title = document.createElement("div");
      title.setAttribute("tower", tower.name);
      title.textContent = tower.shortName;
      towerElm.appendChild(title);
      let difficulty = document.createElement("div");
      difficulty.textContent = tower.difficulty;
      towerElm.appendChild(difficulty);

      this.elements[tower.area].appendChild(towerElm);
      // this.elements[tower.name] = towerElm;
    })
  }

  __loopTower(loopTowers, name) {
    for (let [areaName, towers] of Object.entries(loopTowers)) {
      let finalName = !isNaN(areaName) ? `${name}-${areaName}` : areaName;

      for (let [towerName, tower] of Object.entries(towers)) {
        this.towers.push(new Tower(towerName, finalName, tower.difficulty, tower.badge_id, tower.old_id));
        this.tower_ids[tower.badge_id] = towerName;
        if (tower.old_id) this.tower_ids[tower.old_id] = towerName;
      }

      if (this.areas.includes(finalName)) {
        continue;
      }
      this.areas.push(finalName);
    }
  }

  async loadTowers() {
    let server_towers = await fetch('data/tower_data.json');
    if (!server_towers.ok) {
      showNotification(`Failed to fetch tower_data.json: ${server_towers.status} ${server_towers.statusText}.`, true);
      return;
    }

    this.raw_data = await server_towers.json();
    console.log(this.raw_data);

    this.__loopTower(this.raw_data.rings, 'Ring');
    this.__loopTower(this.raw_data.zones, 'Zone');
  }
}

/**
* @typedef {{
*   old_id: number,
*   badge_id: number
* }} OtherData
*/

class OtherManager {
  /** @type {Object.<string, OtherData>} */
  raw_data;

  async hasBadge(user_id, name) {
    /** @type {OtherData} */
    let badges = this.raw_data[name];
    this.verbose.log(`Checking badge ${name} ({old_id: ${badges.old_id}, badge_id: ${badges.badge_id}) for user ${user_id}`);
    let has = await network.getEarlierBadge(user_id, badges.old_id, badges.badge_id);
    return has.earliest
  }

  badgeToLink(badge_id) {
    return `https://roblox.com/badges/${badge_id}`;
  }

  constructor() {
    this.verbose = new Verbose("OtherManager", "#594832");
    (async () => {
      await this.loadOther();
    })();
  }

  async loadOther() {
    let server_other = await fetch('data/other_data.json');
    if (!server_other.ok) {
      showError(`Failed to fetch tower_data.json: ${server_other.status} ${server_other.statusText}.`, true);
      return;
    }

    this.raw_data = await server_other.json();
    console.log(this.raw_data);
  }
}

let towerManager = new TowerManager();
let otherManager = new OtherManager();
