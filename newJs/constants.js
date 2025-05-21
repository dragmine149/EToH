/*eslint no-undef: "error" */
/*exported DIFFICULTIES, SUB_LEVELS, TOWER_TYPE, pointsFromType */

const DIFFICULTIES = Object.freeze(["Easy", "Medium", "Hard", "Difficult", "Challenging", "Intense", "Remorseless", "Insane", "Extreme", "Terrifying", "Catastrophic"]);
const SUB_LEVELS = Object.freeze([
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
]);
const TOWER_TYPE = Object.freeze({
  Steeple: "Steeple",
  Tower: "Tower",
  Citadel: "Citadel",
  Obelisk: "Obelisk",
  Other: "Other",
  NAT: "Not a Tower"
});

/**
* Gets how many points that type of tower is worth.
* @param {TOWER_TYPE} type The type of tower.
*/
function pointsFromType(type) {
  switch (type) {
    case TOWER_TYPE.Steeple: return 0.5;
    case TOWER_TYPE.Tower: return 1;
    case TOWER_TYPE.Citadel: return 2;
    case TOWER_TYPE.Obelisk: return 3;
    default: return 0;
  }
}
