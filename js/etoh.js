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

/** @type {Towers} */
let towers;

function showError(message) {
  document.getElementById('error_message').innerText = message;
  document.getElementById('errors').hidden = false;
}

async function loadTowers() {
  let server_towers = await fetch('data/tower_data.json');
  if (!server_towers.ok) {
    console.warn(server_towers);
    showError(`Failed to fetch tower_data.json: ${server_towers.status} ${server_towers.statusText}.`);
    return;
  }

  towers = await server_towers.json();
  console.log(towers);
}

loadTowers();

class Tower {
  /** @type {string} */
  name;
  /** @type {number} */
  difficulty;
  /** @type {number} */
  badge;

  get shortName() {
    return this.name.split(' ').map(word => word[0]).join('');
  }
  get difficultyWord() {
    return getDifficulty(this.difficulty);
  }

  constructor(name, difficulty, badge) {
    this.name = name;
    this.difficulty = difficulty;
    this.badge = badge;
  }
}

/**
* Translates the number form into a more readable word form
* @param {number} difficulty The difficulty of the tower
* @returns {string} The word form of the difficulty
*/
function getDifficulty(difficulty) {
  let stage = Math.trunc(difficulty);
  let sub = difficulty % 1;

  let difficulties = ["Easy", "Medium", "Hard", "Difficult", "Challenging", "Intense", "Remorseless", "Insane", "Extreme", "Terrifying", "Catastrophic"];
  let stageWord = difficulties[stage - 1] || "Unknown";

  let subWord = ``;

  if (sub >= 0.89) {
    subWord = "Peak";
  } else if (sub >= 0.78) {
    subWord = "High-Peak";
  } else if (sub >= 0.67) {
    subWord = "High";
  } else if (sub >= 0.56) {
    subWord = "Mid-High";
  } else if (sub >= 0.45) {
    subWord = "Mid";
  } else if (sub >= 0.33) {
    subWord = "Low-Mid";
  } else if (sub >= 0.22) {
    subWord = "Low";
  } else if (sub >= 0.11) {
    subWord = "Bottom-Low";
  } else {
    subWord = "Bottom";
  }


  return `${stageWord} ${subWord}`;
}
