import { tryCatch } from "./utils";
import { type UserTable } from ".";
import { GenericManager } from "./GenericManager";
import { CLOUD_URL, network } from "./network";
import { logs } from "./logs";

class User {
  // The id of the user. Use this where possible
  id: number;
  // The name of the user.
  #name: string;
  // The display name of the user.
  display: string;
  // When this user was last viewed.
  last_viewed: number;
  // Previous names this user has gone by.
  past_names: string[];

  get ui_name() {
    return `${this.display} ${this.display != this.name ? `(@${this.name})` : ``}`;
  }

  get link() {
    return `https://www.roblox.com/users/${this.id}/profile`;
  }

  get profile() {
    return `https://roblox-proxy.dragmine149.workers.dev/users/${this.id}/avatar_headshot.webp?size=48&direct&circular`;
  }

  set name(new_name: string) { this.update_name(new_name); }
  get name() { return this.#name; }

  constructor(id: number, name: string, display: string, past: string[]) {
    this.id = id;
    this.#name = name;
    this.display = display;
    this.past_names = past;
    this.last_viewed = new Date().getTime();
  }

  update_name(new_name: string) {
    this.past_names.push(this.name);
    this.#name = new_name;
  }

  is_user(identifier: string | number) {
    return this.#name == identifier || this.id == identifier;
  }

  load() {
    // DO something...
  }

  unload() {
    // DO something...
  }
}

type UserConstructor<K extends User> = new (id: number, name: string, display: string, past: string[]) => K;

type MinimumUserData = {
  id: number,
  name: string,
  display: string,
}

/**
 * An extension of GenericManager to store user bassed details.
 */
class UserManager<K extends User> extends GenericManager<K, string | number> {
  name!: (item?: string) => string[] | K[];
  id!: (item?: number) => number[] | K[];

  // The database, kinda important for a lot of things.
  #db: UserTable;
  #userClass: UserConstructor<K>;
  #users: K[];

  #current_user?: K;
  /** The current loaded user. Used as a way to automatically load/unload when changing. */
  get current_user() { return this.#current_user; }
  set current_user(user: K | undefined) {
    if (user == undefined) {
      this.#current_user?.unload();
      this.#current_user = undefined;
      return;
    }

    this.#current_user = user;
    this.#current_user.load();
  }

  constructor(database: UserTable, userClass: UserConstructor<K>) {
    super();
    this.addFilter('name', user => [user.name, ...user.past_names].filter((n) => n != undefined));
    this.addFilter('id', user => user.id);

    this.#db = database;
    this.#userClass = userClass;
    this.#users = [];

    this.load_database();
  }

  /**
   * Load user data from the database into memory. This is designed to be done once on startup, however can survive multiple calls.
   * Dispatches the event `user_manager_loaded` once completed. (Can't be bothered to add custom event listener here)
   */
  async load_database() {
    console.info("Loading users from local database");
    let data = await this.#db.toArray()
    data.forEach((user) => {
      const obj = new this.#userClass(user.id, user.name, user.display, user.past);
      this.#users.push(obj);
      // console.log(obj);
      this.addItem(obj);
    });

    dispatchEvent(new Event("user_manager_loaded"));

    // console.info(`Database loaded`);
  }

  /**
   * Checks to see if we already have the user loaded in memory (#users)
   * @param user The user to check.
   * @returns If the user exists and it's the only one in memory.
   */
  #load_user(user?: K) {
    if (!user) return false;

    // pot length should never be > 1 unless roblox did a massive oopsie.
    // "pot" is just the amount of users returned by the user filter.
    // console.log(user);
    const pot = this.#users.filter((u) => u.is_user(user.id ?? user.name));
    // console.log(pot);
    if (pot.length == 1) {
      this.current_user = pot[0];
      return true;
    }

    // Somehow they are not in the pot...
    return false;
  }

  /**
   * Attempts to find the user after going through various checks.
   * @param identifier Minimal Information about the user.
   * @returns The user, or undefined if failed to load.
   */
  async find_user(identifier: string | number) {
    logs.log(`Loading data for ${identifier}`, `user_manager/load`, 0);
    if (this.current_user?.is_user(identifier)) {
      logs.log(`${identifier} is already loaded. Cancelled.`, `user_manager/load`, 100);
      return;
    }

    /// the setter deals with unloading the user.
    this.current_user = undefined;

    if (typeof identifier == 'number') {
      logs.log(`Attempting to search for user by id in database`, `user_manager/load`, 5);
      const id_load = this.id(identifier) as K[];
      const user = id_load[0];
      if (this.#load_user(user)) {
        logs.log(`User loaded via id`, `user_manager/load`, 100);
        return this.current_user;
      }
    }

    if (typeof identifier == 'string') {
      logs.log(`Attempting to search for user by name in database`, `user_manager/load`, 5);
      const id_load = this.name(identifier) as K[];
      const user = id_load[0];
      if (this.#load_user(user)) {
        logs.log(`User loaded via name`, `user_manager/load`, 100);
        return this.current_user;
      }
    }

    logs.log(`Attempting to load user via roblox-proxy`, `user_manager/load`, 10);

    let networkUserRequest = await tryCatch(fetch(new Request(
      `${CLOUD_URL}/users/${identifier}`
    )));

    if (networkUserRequest.error) {
      console.error('Failed to get data from server. Please check your internet and try again. If the issue presits please open an issue on github.');
      return;
    }

    if (!networkUserRequest.data.ok) {
      console.error("Failed to get user data from server. Please check the userID and try again");
      return;
    }


    let userRequest = await tryCatch(networkUserRequest.data.json() as Promise<MinimumUserData>);
    if (userRequest.error) {
      console.error('Failed to parse user data from server. Please try again. If the issue presits please open an issue on github.');
      return;
    }

    logs.log(`Got user data via roblox-proxy`, `user_manager/load`, 50);
    let user = userRequest.data;
    const obj = new this.#userClass(user.id, user.name, user.display, []);
    this.#users.push(obj);
    this.current_user = obj;
    logs.log(`Finish finding user.`, `user_manager/load`, 100);
    return this.current_user;
  }
}

// Export to fix unused declaration warning
export { UserManager, User };
