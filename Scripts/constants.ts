/*eslint no-undef: "error" */

enum DIFFICULTIES {
  Easy, Medium, Hard, Difficult, Challenging, Insane, Remorseless, Intense, Extreme, Terrifying, Catastrophic
}

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

enum TOWER_TYPE {
  Steeple = "Steeple", Tower = "Tower", Citadel = "Citadel", Obelisk = "Obelisk", Mini_Tower = "Mini_Tower", Other = "Other"
}

/**
* Gets how many points that type of tower is worth.
* @param type The type of tower.
*/
function pointsFromType(type: TOWER_TYPE) {
  switch (type) {
    case TOWER_TYPE.Steeple: return 0.5;
    case TOWER_TYPE.Tower: return 1;
    case TOWER_TYPE.Citadel: return 2;
    case TOWER_TYPE.Obelisk: return 3;
    default: return 0;
  }
}

export { DIFFICULTIES, SUB_LEVELS, TOWER_TYPE, pointsFromType }
