/*eslint no-undef: "error"*/

import { GenericManager } from "../Scripts/GenericManager";

// NOTE: by design the badge doesn't store the completed date. The user stores this.
//
// This allows badges to be unique and loaded once, we never really have to touch them again when switching users.
class Badge {
  #name: string;
  #ids: number[];
  #wiki?: URL;

  get ids() { return this.#ids; }
  set ids(_v) { return; }
  get name() { return this.#name; }
  set name(_v) { return; }
  get wiki() { return this.#wiki; }
  set wiki(_v) { return; }

  /**
   * Get the link to the badge. Returns the newest badge id as we assume thats the newest game location.
   * @returns URL to the badge page
   */
  get link() {
    return `https://www.roblox.com/badges/${this.ids[0]}`;
  }

  /**
   * Gets all the links if we have multiple badges.
   * @returns URL to the badges pages.
   */
  get links() {
    return this.ids.map(v => `https://www.roblox.com/badges/${v}`);
  }

  /**
  * Data to show in the name field.
  * @param hover Is this element being hovered.
  */
  get_name_field(hover?: boolean) {
    return hover && this.wiki ? this.name : `<a href="${this.wiki}">${this.name}</a>`;
  }

  /**
  * Data to show in the information field.
  * @param hover Is this element being hovered.
  */
  get_information_field(_hover?: boolean) {
    return `<a href="${this.link}">Badge Link</a>`;
  }

  // /**
  // * How to reference this badge in the search data. key is potential references whilst value is itself.
  // * @param search_data An object of the data to search.
  // */
  // search(search_data: Record<string, string>) {
  //   search_data[this.name] = this.name;
  // }

  /**
  * Create a new badge.
  * @param name The name of the badge.
  * @param ids IDs associated with this badge.
  * @param wiki The link to the wiki page if one exists.
  */
  constructor(name: string, ids: number | number[], wiki?: URL) {
    // this.#addProperty('name', name);
    // this.#addProperty('ids', [].concat(ids));

    this.#name = name;

    if (Array.isArray(ids)) {
      this.#ids = ids;
      return;
    }
    this.#ids = [ids];
    this.#wiki = wiki;
  }
}

class BadgeManager extends GenericManager<Badge, string | number[]> {
  ids!: (item?: number) => number[] | Badge[];
  name!: (item?: string) => string[] | Badge[];

  /**
   * Add a Badge to the manager.
   * @param badge The badge to add.
   */
  addBadge(badge: Badge) {
    if (!(badge instanceof Badge)) {
      throw new Error("Only instances of Badge can be added to BadgeManager.");
    }
    super.addItem(badge);
  }

  /**
  * FIlters out all the badges we have to see if any of them are uncompleted o not.
  * @param completed The completed badge ids.
  * @returns A list of uncompleted basges.
  */
  uncompleted(completed: number[]) {
    // return this.name().map(name => this.name(name)[0]).filter((badge) => badge.ids.some(v => !completed.includes(v)));

    return this.ids()
      .filter((id) => !completed.includes(id as number)) as number[];
  }

  constructor() {
    super();
    this.addFilter('name', badge => badge.name);
    this.addFilter('ids', badge => badge.ids);
  }
}

const badgeManager = new BadgeManager();

export { badgeManager, Badge };
