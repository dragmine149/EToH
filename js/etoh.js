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
 * @typedef {{area_information: AreaInformation, [x: string]: TowerData}} Ring
 */

/**
 * Represents a zone containing towers
 * @typedef {{area_information: AreaInformation, [x: string]: TowerData}} Zone
 */

/**
* Represents a zone containing towers
* @typedef {{area_information: AreaInformation, [x: string]: TowerData}} Event
*/

/**
 * @typedef {{
 *   rings: Object.<string, Ring>,
 *   zones: Object.<string, Zone>,
 *   events: Object.<string, Event>
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
    return this.name.split(' ').map(word => word.toLowerCase()).map(word => (word == 'of' || word == 'and') ? word[0] : word[0].toUpperCase()).join('');
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
    { threshold: 0.34, name: "Low-Mid" },
    { threshold: 0.23, name: "Low" },
    { threshold: 0.12, name: "Bottom-Low" },
    { threshold: 0.01, name: "Bottom" },
    { threshold: 0.00, name: "Baseline" }
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
      document.dispatchEvent(new Event('towers_loaded'));
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
    let subWord = this.subLevels.find(level => sub >= level.threshold)?.name || "Bottom";

    return `${subWord} ${stageWord}`;
  }

  __create_elements() {
    this.areas.forEach(areaName => {
      /** @type {HTMLDivElement} */
      let areaElement = document.getElementById("category").cloneNode(true);
      areaElement.id = `area-${areaName}`;
      areaElement.classList.add("area");
      areaElement.hidden = false;
      /** @type {HTMLSpanElement} */
      let title = areaElement.querySelector("[tag='title']");
      title.innerText = areaName;

      this.elements[areaName] = areaElement;
      this.__areaElm.appendChild(areaElement);
    })

    // this.difficulties.forEach(difficulty => {
    //   let difficultyElement = document.createElement('div');
    //   difficultyElement.id = `difficulty-${difficulty}`;
    //   difficultyElement.classList.add('difficulty');
    //   this.__areaElm.appendChild(difficultyElement);
    //   this.elements[difficulty] = difficultyElement;
    // });

    this.towers.forEach(tower => {
      let towerElm = document.createElement("tr");
      towerElm.setAttribute("tower", tower.name);
      let nameElm = document.createElement("td");
      let difficultyElm = document.createElement("td");

      nameElm.innerText = tower.shortName;
      // difficultyElm.innerText = tower.difficulty;
      let difficultyWord = this.difficulties[Math.trunc(tower.difficulty) - 1];
      difficultyElm.innerText = difficultyWord;
      difficultyElm.classList.add('difficulty', difficultyWord);

      towerElm.onmouseover = () => {
        nameElm.innerText = tower.name;
        difficultyElm.innerText = `${this.getDifficulty(tower.difficulty)} (${tower.difficulty})`;
      }
      towerElm.onmouseleave = () => {
        nameElm.innerText = tower.shortName;
        difficultyElm.innerText = difficultyWord;
      }

      towerElm.appendChild(nameElm);
      towerElm.appendChild(difficultyElm);

      this.elements[tower.area].querySelector('table').appendChild(towerElm);
    });

    ui.updateMainUi(true);

    let tallest = 0;

    Object.values(this.elements).forEach(element => {
      // Temporarily hover each tower element
      element.querySelectorAll('[tower]').forEach(tower => {
        const mouseOverEvent = new Event('mouseover');
        tower.dispatchEvent(mouseOverEvent);
      });

      // Set fixed width
      element.style.width = element.clientWidth + 'px';
      tallest = Math.max(tallest, element.clientHeight);

      // Unhover everything
      element.querySelectorAll('[tower]').forEach(tower => {
        const mouseLeaveEvent = new Event('mouseleave');
        tower.dispatchEvent(mouseLeaveEvent);
      });

      let completedElm = document.createElement('tr');
      completedElm.classList.add('total_completed');
      let title = document.createElement('td');
      title.innerText = 'Completed';
      let count = document.createElement('td');
      count.setAttribute('counter', '0');
      count.setAttribute('total', element.querySelectorAll('[tower]').length);
      count.innerText = `0/${element.querySelectorAll('[tower]').length}`;
      completedElm.appendChild(title);
      completedElm.appendChild(count);
      element.querySelector('table').appendChild(completedElm);
    });

    Object.values(this.elements).forEach(element => element.style.height = tallest + 'px');

    ui.updateMainUi(false);
  }

  __loopTower(loopTowers, name) {
    for (let [areaName, towers] of Object.entries(loopTowers)) {
      if (areaName == 'area_information') {
        continue;
      }

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
      ui.showError(`Failed to fetch tower_data.json: ${server_towers.status} ${server_towers.statusText}.`, true);
      return;
    }

    this.raw_data = await server_towers.json();
    console.log(this.raw_data);

    this.__loopTower(this.raw_data.rings, 'Ring');
    this.__loopTower(this.raw_data.zones, 'Zone');
    this.__loopTower(this.raw_data.events, 'Event');
  }

  /**
  * Prepare the ui for loading data
  * @param {{id: number, name: string, ui: string, played: boolean}} user The user to load.
  */
  prepareUI(user) {
    ui.updateLoadedUser(user.name, user.ui);
    ui.updateMainUi(true);

    Object.values(this.elements).forEach(element => {
      element.querySelectorAll('td').forEach(td => td.classList.remove('completed'));
      element.querySelector('[counter]').setAttribute('counter', 0);
    });
  }

  shown = [];

  /**
  * Show the tower on the UI as completed.
  * @param {{badgeId: number, date: number}} tower_details The tower to show as completed.
  */
  showTower(tower_details) {
    // console.log(tower_details);
    let tower = this.towers.filter(t => t.badge == tower_details.badgeId || t.old_badge == tower_details.badgeId)[0];

    if (this.shown.includes(tower.name)) {
      console.log('Tower already shown: ', tower.name);
      return;
    }

    // if (!tower) return;
    this.elements[tower.area].querySelector(`[tower="${tower.name}"] td`).classList.add('completed');

    // Gets the counter values and display the percent alongside the tower completion count of that area.
    let counter = this.elements[tower.area].querySelector(`[counter]`);
    counter.setAttribute('counter', parseInt(counter.getAttribute('counter')) + 1);
    let percentage = parseInt(counter.getAttribute('counter')) / parseInt(counter.getAttribute('total')) * 100;
    percentage = percentage.toFixed(1);
    counter.innerText = `${counter.getAttribute('counter')}/${counter.getAttribute('total')} (${percentage}%)`;
    counter.parentElement.style.background = `linear-gradient(to right, var(--completed) ${percentage}%, rgba(0, 0, 0, 0) ${percentage}%)`;

    this.shown.push(tower.name);
  }
}

let towerManager = new TowerManager();
