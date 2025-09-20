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

function typeFromNumber(num: number) {
  switch (num) {
    case 0: return TOWER_TYPE.Mini_Tower;
    case 1: return TOWER_TYPE.Steeple;
    case 2: return TOWER_TYPE.Tower;
    case 3: return TOWER_TYPE.Citadel;
    case 4: return TOWER_TYPE.Obelisk;
    default: return TOWER_TYPE.Other;
  }
}


type ServerTower = {
  name: string;
  difficulty: number;
  badges: number[];
  type?: string;
};

type Difficulties = {
  easy?: number;
  medium?: number;
  hard?: number;
  difficult?: number;
  challenging?: number;
  intense?: number;
  remorseless?: number;
  insane?: number;
  extreme?: number;
  terrifying?: number;
  catastrophic?: number;
};


interface AreaRequirements {
  difficulties: Difficulties;
  points: number;
};


type ServerDifficulties = {
  e?: number;
  m?: number;
  h?: number;
  d?: number;
  c?: number;
  i?: number;
  r?: number;
  s?: number;
  x?: number;
  t?: number;
  a?: number;
};


interface ServerAreaRequirements {
  ds: ServerDifficulties;
  p: number;
};

type ServerAreas = {
  /** Name of the area. */
  n: string;
  r: ServerAreaRequirements
  /** If this area is a sub area, and if so. The name of the parent area. */
  s?: string;
  /** The towers in this area. */
  t: string[];
};
type ServerTowers = {
  areas: {
    temporary: ServerAreas[];
    permanent: ServerAreas[];
    other: ServerAreas[];
  };
};
type ServerOther = {
  name: string;
  category: string;
  badges: number[];
};
type ServerOtherParent = {
  data: ServerOther[];
};

export { DIFFICULTIES, SUB_LEVELS, TOWER_TYPE, pointsFromType }
export type { ServerAreas, ServerDifficulties, ServerOther, ServerOtherParent, ServerTower, ServerTowers, AreaRequirements };
