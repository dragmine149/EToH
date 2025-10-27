/*eslint no-undef: "error" */
enum DIFFICULTIES {
  Easy, Medium, Hard, Difficult, Challenging, Intense, Remorseless, Insane, Extreme, Terrifying, Catastrophic
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


interface ServerTower {
  name: string;
  difficulty: number;
  badges: number[];
  type?: string;
}

interface Difficulties {
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
}


interface AreaRequirements {
  difficulties: Difficulties;
  points: number;
};


interface ServerDifficulties {
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
}


interface ServerAreaRequirements {
  ds: ServerDifficulties;
  p: number;
};

interface ServerAreas {
  /** Name of the area. */
  n: string;
  r: ServerAreaRequirements
  /** If this area is a sub area, and if so. The name of the parent area. */
  s?: string;
  /** The towers in this area. */
  t: string[];
}
interface ServerTowers {
  areas: {
    temporary: ServerAreas[];
    permanent: ServerAreas[];
    other: ServerAreas[];
  };
}
interface ServerOther {
  name: string;
  category: string;
  badges: number[];
}
interface ServerOtherParent {
  data: ServerOther[];
}

export { DIFFICULTIES, SUB_LEVELS }
export type { ServerAreas, ServerDifficulties, ServerOther, ServerOtherParent, ServerTower, ServerTowers, AreaRequirements };
