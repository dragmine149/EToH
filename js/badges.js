/**
* @typedef {{
  badge_id: number
  old_id?: number
}} badgeIds The ids for this badge.
*/

class Badge {
  old_id;
  badge_id;
  name;
  is_tower;

  /**
   * @param {number} old_id The badge id of the old game.
   * @param {number} badge_id The badge id of the new game.
   * @param {string} name The name of the badge.
   * @param {boolean} is_tower Whether the badge is a tower badge.
   */
  constructor(old_id, badge_id, name, is_tower) {
    this.old_id = old_id;
    this.badge_id = badge_id;
    this.name = name;
    this.is_tower = is_tower;
  }

  /**
   * @returns {string} The link to the (new) badge.
   */
  get link() {
    return `https://roblox.com/badges/${this.badge_id}`;
  }
}

class BadgeManager {
  badges = [];

  constructor() {
    this.__loadOtherBadgesFromServer();
    this.__loadTowerBadgesFromServer();
    this.verbose = new Verbose('BadgeManager', '#00FA9A');
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
      ui.showError(`Failed to parse other_data.json: ${data.error}`, true);
      return;
    }

    Object.entries(data.data).forEach((entry) => {
      this.badges.push(new Badge(entry[1].old_id, entry[1].badge_id, entry[0]));
    });
  }

  async __loadTowerBadgesFromServer() {
    let server_tower = await fetch('data/tower_data.json');
    if (!server_tower.ok) {
      ui.showError(`Failed to fetch tower_data.json: ${server_tower.status} ${server_tower.statusText}.`, true);
      return;
    }

    /** @type {{data: Towers | null, error: Error | null}} */
    let data = await tryCatch(server_tower.json());

    if (data.error) {
      ui.showError(`Failed to parse other_data.json: ${data.error}`, true);
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

  async hasBadge(user_id, badge_name) {
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
    this.verbose.log(`Looking for badge: ${badge}`);

    if (typeof badge === 'number') {
      this.verbose.log(`number`);
      return this.badges.find(predict_badge => predict_badge.badge_id === badge || predict_badge.old_id === badge);
    } else if (typeof badge === 'string') {
      this.verbose.log(`string`);
      return this.badges.find(predict_badge => predict_badge.name === badge);
    }
    return new Badge(0, 0, "unknown", false);
  }
}

let badgeManager = new BadgeManager();
