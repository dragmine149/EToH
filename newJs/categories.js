class Badge {
  /** @type string */
  name = "";
  /** @type number The id of the badge */
  primaryId = 0;
  /** @type number[] Other ids of the same badge. This is useful for when a game moves locations (hence new badges) */
  ids = [];
  information = {};

  /**
   * Get the link to the badge. Returns the primary one by default as we assume the others are impossible to get.
   * @returns {string} URL to the badge page
   */
  get link() {
    return `https://www.roblox.com/badges/${this.primaryId}`;
  }

  /**
  * Checks to see if we contain a specific badge id.
  * @param {number} filterBadge The badge to check for (in any filter type thing)
  * @returns
  */
  filterChecker(filterBadge) {
    return this.primaryId === filterBadge || this.ids.includes(filterBadge);
  }

  /**
  * Makes a new badge to store information about badges.
  * @param {string} name The name of the Badge
  * @param {number} id The primary id of the badge. Pioritised and used first.
  */
  constructor(name, id) {
    this.name = name;
    this.primaryId = id;
  }

  /**
  * Adds a set of ids to the list.
  * @param {...number} ids The other badge ids
  */
  addIds(...ids) {
    this.ids.push(...ids);
  }

  /**
  * Some badges contain more information than we can cope with. Hence expand a badge here.
  * @param {string} key
  * @param {any} value
  */
  addInformation(key, value) {
    this.information[key] = value;
  }

  /**
  * Retrieves the information previously stored.
  * @param {string} key
  * @returns
  */
  getInformation(key) {
    return this.information[key];
  }
}

class Category {
  /** @type Category[] */
  subCategories = [];
  /** @type Badge[] */
  badges = [];
  information = {};

  filterChecker(filterBadge) {
    return this.badges.some(badge => badge.filterChecker(filterBadge)) || this.subCategories.some(subCategory => subCategory.filterChecker(filterBadge));
  }

  constructor(name) {
    this.name = name;
  }

  addBadge(badge) {
    this.badges.push(badge);
  }

  addSubCategory(category) {
    this.subCategories.push(category);
  }

  addInformation(key, value) {
    this.information[key] = value;
  }

  getInformation(key) {
    return this.information[key];
  }
}

let categories = [];
