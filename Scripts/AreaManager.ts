import { GenericManager } from "../Scripts/GenericManager";
import { DIFFICULTIES } from "../Scripts/constants";

interface AreaRequirements {
  difficulties: ServerDifficulties;
  points: number;
};

class Area {
  /** @type The name of this area */
  name: string;
  /** @type The parent area. */
  parent?: string;

  /** @type The requirements to access that area */
  requirements: AreaRequirements;

  constructor(name: string, parent: string, requirements: AreaRequirements) {
    this.name = name;
    this.parent = parent;
    this.requirements = requirements;
  }
}

class AreaManager extends GenericManager<Area, string> {
  parent!: (item: string) => string[] | Area[];
  name!: (item: string) => string[] | Area[];
  difficulties!: (item: string) => string[] | Area[];

  /**
   * Add an Area to the manager.
   * @param area The area to add.
   */
  addArea(area: Area) {
    if (!(area instanceof Area)) {
      throw new Error("Only instances of Area can be added to AreaManager.");
    }
    super.addItem(area);
  }

  constructor() {
    super();
    this.addFilter("parent", area => area.parent || "");
    this.addFilter("name", area => area.name);

    Object.entries(DIFFICULTIES).forEach((diff) => {
      this.addFilter(diff[0], area => area.requirements.difficulties[diff[0].toLowerCase()])
    })
  }
}

const areaManager = new AreaManager();

export { areaManager, Area };
