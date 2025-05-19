/*global GenericManager, DIFFICULTIES */
/*eslint no-undef: "error" */
/*exported Area, areaManager */

class Area {
  /** @type {string} The name of this area */
  name;
  /** @type {string?} The parent area. */
  parent;

  /** @type {{
    difficulties: ServerDifficulties,
    points: number
  }} The requirements to access that area */
  requirements;

  constructor(name, parent, requirements) {
    this.name = name;
    this.parent = parent;
    this.requirements = requirements;
  }
}

class AreaManager extends GenericManager {
  /**
   * Add an Area to the manager.
   * @param {Area} area The area to add.
   */
  addArea(area) {
    if (!(area instanceof Area)) {
      throw new Error("Only instances of Area can be added to AreaManager.");
    }
    super.addItem(area);
  }

  constructor() {
    super();
    this.addFilter("parent", area => area.parent);
    this.addFilter("name", area => area.name);
    this.addFilter("points", area => area.requirements.points);
    DIFFICULTIES.forEach((diff) => {
      this.addFilter(diff, area => area.requirements.difficulties[diff.toLowerCase()]);
    })
  }
}

let areaManager = new AreaManager();
