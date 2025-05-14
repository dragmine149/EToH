/*global GenericManager*/
/*eslint no-undef: "error"*/
/*exported badgeManager, Badge */

class Badge {
  /** @type {number[]} The ids that are associated with the badge. As games will sometimes "move", we need a list to store all possibilities. Sorted as newest -> oldest */
  ids;
  /** @type {String} The name of the badge, doesn't have to match the one provided by the game, just has to be something useable. */
  name;

  /**
   * Get the link to the badge. Returns the newest badge id as we assume thats the newest game location.
   * @returns {string} URL to the badge page
   */
  get link() {
    return `https://www.roblox.com/badges/${this.ids[0]}`;
  }

  /**
  * Adds a new uneditable property to the object.
  * @param {String} name The name of this property.
  * @param {any} value The value to assign to this property.
  */
  __addProperty(name, value) {
    if (this[name]) return; // don't readd it.

    // Custom getter and setter functions. These are meant to not set and always get. Badge data is never going to update live unless a system is implemented, these help with that.
    Object.defineProperty(this, name, {
      get: function () { return this[`__${name}`] },
      set: function () { }
    });
    this[`__${name}`] = value;
  }

  /**
  * Create a new badge.
  * @param {String} name The name of the badge.
  * @param {number[]} ids IDs associated with this badge.
  */
  constructor(name, ids) {
    this.__addProperty('name', name);
    this.__addProperty('ids', ids);
  }
}

class BadgeManager extends GenericManager {
  /**
   * Add a Badge to the manager.
   * @param {Badge} badge The badge to add.
   */
  addBadge(badge) {
    if (!(badge instanceof Badge)) {
      throw new Error("Only instances of Badge can be added to BadgeManager.");
    }
    super.addItem(badge);
  }

  constructor() {
    super();
    this.addFilter('names', badge => badge.name);
  }
}


let badgeManager = new BadgeManager();
