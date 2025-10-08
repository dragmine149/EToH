/*eslint no-undef: "error"*/

import { GenericManager } from "../Scripts/GenericManager";

// The reason why this badge / category can not be claimed.
//
// The lock state of a badge should never change unless external data says so.
enum Lock {
  // The badge is unlocked for the taking. (aka you can get it at anytime pretty much)
  Unlocked,
  // The badge is unlocked but only for a limited time.
  Limited,
  // The badge / category was only available for a limited time.
  Temporary,
  // The badge / category requires another badge / category to be unlocked first.
  //
  // NOTE: The UI will automatically sort this out. Badge object should not change.
  Another
}

// NOTE: by design the badge doesn't store the completed date. The user stores this.
//
// This allows badges to be unique and loaded once, we never really have to touch them again when switching users.
class Badge {
  #name: string;
  #ids: number[];
  #wiki?: URL;
  #lock_type: Lock;
  #lock_reason?: string;

  /** A readonly list of ids relating to this badge. */
  get ids() { return this.#ids; }
  set ids(_v) { return; }
  /** A readonly name associated with this badge. */
  get name() { return this.#name; }
  set name(_v) { return; }
  /** A readonly wiki link associated with this badge. */
  get wiki() { return this.#wiki; }
  set wiki(_v) { return; }

  /** The overlying type of why this badge is unobtainable. */
  get lock_type() { return this.#lock_type; }
  set lock_type(_v) { return; }

  /** The reason to why this badge is unobtainable. More in depth than lock_type. */
  get lock_reason() { return this.#lock_reason; }
  set lock_reason(_v) { return; }

  /** Returns the primary id of this badge. */
  get id() {
    return this.ids[0];
  }

  /**
   * Get the link to the badge. Returns the newest badge id as we assume thats the newest game location.
   * @returns URL to the badge page
   */
  get link() {
    return `https://www.roblox.com/badges/${this.id}`;
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
    return hover && this.wiki ? `<a href="${this.wiki}">${this.name}</a>` : this.name;
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
  constructor(name: string, ids: number | number[], lock_type: Lock, wiki?: URL, lock_reason?: string) {
    // this.#addProperty('name', name);
    // this.#addProperty('ids', [].concat(ids));

    this.#name = name;
    this.#lock_type = lock_type;
    this.#wiki = wiki;
    this.#lock_reason = lock_reason;

    if (Array.isArray(ids)) {
      this.#ids = ids;
      return;
    }
    this.#ids = [ids];
  }
}

class BadgeManager<B extends Badge, T> extends GenericManager<B, string | number[] | Lock | T> {
  /** Given an id, returns a list of badges with that id. Given nothing, returns a list of ids. */
  ids!: { (): number[], (item?: number): B[] };
  /** Given a name, returns a list of badges with that name. Given nothing, returns a list of badge names. */
  name!: { (): string[], (item?: string): B[] };
  /** Given a lock, returns a list of badges with that lock. Given nothing, returns a list of lock. */
  lock!: { (): Lock[], (item?: Lock): B[] };

  /**
   * Add a Badge to the manager.
   * @param badge The badge to add.
   */
  addBadge(badge: B) {
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
    this.addFilter('lock', badge => badge.lock_type);
  }
}

export { BadgeManager, Badge, Lock };
