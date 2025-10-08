import { GenericManager } from "../Scripts/GenericManager";
import { DIFFICULTIES, AreaRequirements } from "../Scripts/constants";
import { Lock } from "./BadgeManager";
import { Category } from "./Etoh";

class Area {
  /** The name of this area */
  name: string;
  /** The parent area. */
  parent?: string;
  /** The requirements to access that area */
  requirements: AreaRequirements;
  /** The type of area that this is */
  category: Category;
  /** The overlaying type as to why this area is locked. Being locked will also make all children locked */
  lock_type: Lock;
  /** A more indepth reason as to why this area is locked. */
  lock_reason?: string;

  constructor(name: string, parent: string | undefined, requirements: AreaRequirements, category: Category, lock_type: Lock, lock_reason?: string) {
    this.name = name;
    this.parent = parent;
    this.requirements = requirements;
    this.category = category;
    this.lock_type = lock_type;
    this.lock_reason = lock_reason;
  }
}

class AreaManager extends GenericManager<Area, string | Category | Lock> {
  /** Given a parent, returns a list of areas with that parent. Given nothing, returns a list of parents. */
  parent!: { (): string[], (item: string): Area[] };
  /** Given a name, returns a list of areas with that name. Given nothing, returns a list of area names. */
  name!: { (): string[], (item?: string): Area[] };
  /** Given a category, returns a list of areas with that category. Given nothing, returns a list of categories. */
  category!: { (): Category[], (item?: Category): Area[] };
  /** Given a lock, returns a list of areas with that lock. Given nothing, returns a list of locks. */
  lock!: { (): Lock[], (item?: Lock): Area[] };

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
    this.addFilter("category", area => area.category);
    this.addFilter("lock", area => area.lock_type);

    Object.entries(DIFFICULTIES).forEach((diff) => {
      this.addFilter(diff[0], area => area.requirements.difficulties[diff[0].toLowerCase()] || 0)
    })
  }
}

const areaManager = new AreaManager();

export { areaManager, Area };
