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
    return `https://www.roblox.com/badge/${this.primaryId}`;
  }

  filterChecker(filterBadge) {
    return this.primaryId === filterBadge || this.ids.includes(filterBadge);
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
}

let categories = [];
