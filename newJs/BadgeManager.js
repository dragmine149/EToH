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
    if (this[name]) return;
    Object.defineProperty(this, name, {
      get: function () { return this[`__${name}`] },
      set: function (v) { return false }
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

class BadgeManager {
  __badges = [];
  /** @type {{filter: String, callback: (b: Badge) => any}[]} A list of filters to apply to badges */
  __filters = [];

  /**
  * Get an item from a map. Returns the map keys if no item is defined.
  * @param {BadgeManager} self
  * @param {Map} map
  * @param {any} item
  */
  __mapGetter(self, map, item) {
    if (item == null || item == undefined) {
      return Array.from(map.keys());
    }

    /** @type {number[]} */
    let indexes = map.get(item);
    if (indexes == undefined) return undefined;
    return indexes.map((index) => self.__badges[index]);
  }

  /**
  * Add a badge to the map.
  * @param {BadgeManager} self
  * @param {Map} map
  * @param {any} item
  * @param {number} value
  */
  __mapSetter(self, map, item, value) {
    let data = map.get(item);
    if (data == undefined) {
      data = [];
    }
    data.push(value);
    map.set(item, data);
  }

  /**
  * Add a badge to the manager of badges.
  * @param {Badge} badge The badge to add.
  */
  addBadge(badge) {
    let index = this.__badges.push(badge);

    this.__filters.forEach((filter) => {
      let key = filter.callback(badge);
      if (key == undefined || key == null) return;

      this.__mapSetter(this, this[`__${filter.filter}`], key, index - 1);
    });
  }

  /**
  * Makes a new map to have a shortcut way of getting tower data. Data can now be retrieved using `class['filter']('test')`
  * @param {String} filter The thing to store in order to filter badges
  * @param {(b: Badge) => any} callback What gets stored in the map for quick access to the badges
  */
  addFilter(filter, callback) {
    this.__filters.push({
      filter, callback: callback
    })
    this[`__${filter}`] = new Map();
    this[filter] = this.__mapGetter.bind(null, this, this[`__${filter}`]);

    if (this.__badges.length > 0) {
      this.__badges.forEach((badge, index) => {
        let key = callback(badge);
        if (key == undefined || key == null) return;

        this[`__${filter}`].set(key, index);
      })
    }
  }

  constructor() {
    this.addFilter('names', b => b.name);
  }
}

let badgeManager = new BadgeManager();
