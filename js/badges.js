/**
* @typedef {{
  badge_id: number
  old_id?: number
}} badgeIds The ids for this badge.
*/

class Badge {
  old_id;
  new_id;
  name;
  is_tower;

  /**
   * @param {number} old_id The badge id of the old game.
   * @param {number} new_id The badge id of the new game.
   * @param {string} name The name of the badge.
   * @param {boolean} is_tower Whether the badge is a tower badge.
   */
  constructor(old_id, new_id, name, is_tower) {
    this.old_id = old_id;
    this.new_id = new_id;
    this.name = name;
    this.is_tower = is_tower;
  }

  /**
   * @returns {string} The link to the (new) badge.
   */
  get link() {
    return `https://roblox.com/badges/${this.new_id}`;
  }
}

class BadgeManager {
  badges = [];

  constructor() {
    this.__loadOtherBadgesFromServer();
    this.__loadTowerBadgesFromServer();
  }

  async __loadOtherBadgesFromServer() {
    let server_other = await fetch('data/other_data.json');
    if (!server_other.ok) {
      showError(`Failed to fetch tower_data.json: ${server_other.status} ${server_other.statusText}.`, true);
      return;
    }

    /** @type {{data: Object.<string, badgeIds> | null, error: Error | null}} */
    let data = await tryCatch(server_other.json());

    if (data.error) {
      showError(`Failed to parse other_data.json: ${data.error}`, true);
      return;
    }

    Object.entries(data.data).forEach((entry) => {
      this.badges.push(new Badge(entry[1].old_id, entry[1].badge_id, entry[0]));
    });
  }

  async __loadTowerBadgesFromServer() {
    let server_tower = await fetch('data/tower_data.json');
    if (!server_tower.ok) {
      showError(`Failed to fetch tower_data.json: ${server_tower.status} ${server_tower.statusText}.`, true);
      return;
    }

    /** @type {{data: Towers | null, error: Error | null}} */
    let data = await tryCatch(server_tower.json());

    if (data.error) {
      showError(`Failed to parse other_data.json: ${data.error}`, true);
      return;
    }

    Object.entries(data.data.rings).forEach((entry) => {
      Object.entries(entry[1]).forEach((subEntry) => {
        this.badges.push(new Badge(subEntry[1].old_id, subEntry[1].badge_id, subEntry[0], true));
      });
    });
    Object.entries(data.data.rings).forEach((entry) => {
      Object.entries(entry[1]).forEach((subEntry) => {
        this.badges.push(new Badge(subEntry[1].old_id, subEntry[1].badge_id, subEntry[0], true));
      });
    });
  }

  async hadBadge(user_id, badge_name) {
    let badge = this.badges.find(badge => badge.name === badge_name);
    if (!badge) {
      return 0;
    }

    this.verbose.log(`Checking badge ${badge_name} ({old_id: ${badge.old_id}, badge_id: ${badge.badge_id}) for user ${user_id}`);
    let has = await network.getEarlierBadge(user_id, badge.old_id, badge.badge_id);
    return has.earliest
  }

  /** Returns the information about a badge.
  * @type {number | string} badge
  * @returns {Badge | null} the found badge (or null if not found)
  */
  badge(badge) {
    if (typeof badge === 'number') {
      return this.badges.find(badge => badge.id === badge || badge.old_id === badge);
    } else if (typeof badge === 'string') {
      return this.badges.find(badge => badge.name === badge);
    }
    return null;
  }
}

let badgeManager = new BadgeManager();
