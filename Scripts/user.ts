import { Verbose } from "./verbose.mjs";
import { tryCatch } from "./utils";
import { type UserTable } from ".";
import { GenericManager } from "./GenericManager";
import { CLOUD_URL, network } from "./network";
import type { EntityTable, Dexie } from "dexie";
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
    return `${this.display} ${this.display != this.name ? `(@${this.name})}` : ``}`;
  }

  get link() {
    return `https://www.roblox.com/users/${this.id}/profile`;
  }

  set name(new_name: string) { this.update_name(new_name); }
  get name() { return this.#name; }

  get database() {
    return {
      id: this.id,
      name: this.#name,
      display: this.display,
      past: this.past_names,
      last: this.last_viewed
    }
  }

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
    this.#name == identifier || this.id == identifier;
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

class UserManager<K extends User> extends GenericManager<K, string | number> {
  name!: (item: string) => string[] | K[];
  id!: (item: number) => number[] | K[];

  // The database, kinda important for a lot of things.
  #db: UserTable;
  #userClass: UserConstructor<K>;
  #users: K[];

  #current_user?: K;
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

  #verbose: Verbose;

  constructor(database: UserTable, userClass: UserConstructor<K>) {
    super();
    this.addFilter('name', user => [user.name, ...user.past_names]);
    this.addFilter('id', user => user.id);

    this.#db = database;
    this.#userClass = userClass;
    this.#verbose = new Verbose("UserManager", '#afe9ca');
    this.#users = [];

    this.load_database();
  }

  async load_database() {
    this.#verbose.info("Loading users from local database");
    (await this.#db.toArray()).forEach((user) => {
      const obj = new this.#userClass(user.id, user.name, user.display, user.past);
      this.#users.push(obj);
      this.addItem(obj);
    })
  }

  load_user(user?: K) {
    if (!user) return false;

    // pot length should never be > 1 unless roblox did a massive oopsie.
    const pot = this.#users.filter((u) => u.is_user(user.id ?? user.name));
    if (pot.length == 1) {
      this.current_user = pot[0];
      return true;
    }

    // Somehow they are not in the pot...
    return false;
  }

  load_network_user(user: MinimumUserData) {
    const obj = new this.#userClass(user.id, user.name, user.display, []);
    this.#users.push(obj);
    this.current_user = obj;
  }

  /**
   * Attempts to find the user after going through various checks.
   * @param identifier Minimal Information about the user.
   * @returns A user
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
      if (this.load_user(user)) {
        logs.log(`User loaded via id`, `user_manager/load`, 100);
        return;
      }
    }

    if (typeof identifier == 'string') {
      logs.log(`Attempting to search for user by name in database`, `user_manager/load`, 5);
      const id_load = this.name(identifier) as K[];
      const user = id_load[0];
      if (this.load_user(user)) {
        logs.log(`User loaded via name`, `user_manager/load`, 100);
        return;
      }
    }

    logs.log(`Attempting to load user via roblox-proxy`, `user_manager/load`, 10);

    let networkUserRequest = await tryCatch(network.retryTilResult(new Request(
      `${CLOUD_URL}/users/${(this.id ?? this.name)}`
    )));

    if (networkUserRequest.error) {
      this.#verbose.error('Failed to get data from server. Please check your internet and try again. If the issue presits please open an issue on github.');
      return;
    }

    let userRequest = await tryCatch(networkUserRequest.data.json() as Promise<MinimumUserData>);
    if (userRequest.error) {
      this.#verbose.error('Failed to parse user data from server. Please try again. If the issue presits please open an issue on github.');
      return;
    }

    logs.log(`Got user data via roblox-proxy`, `user_manager/load`, 50);
    this.load_network_user(userRequest.data);
    logs.log(`Finish finding user.`, `user_manager/load`, 100);
  }
}

// Export to fix unused declaration warning
export { UserManager, User };
