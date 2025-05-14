/*global Verbose, CLOUD_URL, tryCatch, network, GenericManager, etohDB */
/*eslint no-undef: "error"*/
/*exported User, UserManager */

class User {
  /** @type {number} */
  id;

  /** @type {string[]} */
  past = [];

  /** @type {string} */
  name;

  /** @type {string} */
  display;

  /** @type {number} */
  // If we run out of space, we'll remove the users that have not been viewed in a while first.
  last = 0;

  get ui_name() {
    if (this.display != null) {
      return `${this.display} (@${this.name})`;
    }
    return this.name
  }

  get link() {
    return `https://www.roblox.com/users/${this.id}/profile`;
  }

  get database() {
    return {
      id: this.id,
      name: this.name,
      display: this.display,
      past: this.past,
      last: this.last
    }
  }

  constructor(user_data) {
    this.verbose = new Verbose('user', 'orangered');

    if (typeof user_data === 'object' && user_data !== null) {
      this.id = user_data.id;
      this.name = user_data.name;
      this.display = user_data.display;
      this.past = user_data.past;
      this.last = user_data.last;
      return;
    }
  }

  /**
  * Create a new user object. Returns depending on what happened.
  * @param {{id: number, name: string, display: string, past: string[], last: number}} user_data
  * @param {any} db Database to check if the user exists under a different entry instead.
  * @returns {Promise<User|number|null>} User = valid. Number = in database under different user. Null = server/internal error.
  */
  static async create(user_data, db) {
    let user = new User(user_data);
    if (user.id) return user;

    user.name = user_data;
    let nan = Number(user_data);
    if (!Number.isNaN(nan)) {
      user.id = nan;
      user.name = undefined;
    }
    let result = await user.updateDetails(db);
    user.verbose.info(`Received: ${result} from request server data.`);
    if (!Number.isNaN(result) && result !== true) {
      user.verbose.info(`Is number!`);
      return result;
    }
    if (result !== true) {
      user.verbose.info(`Is not true!`)
      return null;
    }

    user.verbose.info(`Is user!`);
    return user;
  }

  async updateDetails(db) {
    this.verbose.log('Attempting to update user details', {
      id: this.id, past: this.past, name: this.name, display: this.display
    });

    // if we call this function, although we might already have the user details. Update them anyway, in case of display/user name changes.

    let networkUserRequest = await tryCatch(network.retryTilResult(new Request(
      `${CLOUD_URL}/users/${(this.id ?? this.name)}`
    )));

    if (networkUserRequest.error) {
      this.verbose.error('Failed to get data from server. Please check your internet and try again. If the issue presits please open an issue on github.');
      return;
    }

    let userRequest = await tryCatch(networkUserRequest.data.json());
    if (userRequest.error) {
      this.verbose.error('Failed to parse user data from server. Please try again. If the issue presits please open an issue on github.');
      return;
    }

    /** @type {{id: number, name: String, display: String}} */
    let userData = userRequest.data;

    if (!this.id && db) {
      this.verbose.debug(`Checking database to see if we already have ${userData.id} in the database`);
      let potential = await etohDB.users.get({ id: userData.id });
      if (potential) {
        this.verbose.debug(`Found user id, returning to use that user instead.`);
        return userData.id;
      }
      this.verbose.info(`We do not, hence saving data.`);
    }

    if (this.id && this.id != userData.id) {
      this.verbose.error(`Id mismatch! (${this.id} != ${userData.id}.`);
      return;
    }

    // this way, we aren't storing unnecessary data by getting all of the past names.
    if (userData.name != this.name) {
      this.verbose.info(`User has new name: ${userData.name}, putting ${this.name} onto past list`);
      this.past.push(this.name);
    }

    this.display = userData.display;
    this.id = userData.id;
    this.name = userData.name;
    return true;
  }
}


// Note: Current assumption is down to using Dexie w/ a table called `users`
class UserManager extends GenericManager {
  async findUser(identifier) {
    // store the current user as we've finished with them.
    if (this.current_user != null) {
      this.verbose.debug(identifier, this.current_user);
      if (this.current_user.id == identifier || this.current_user.name == identifier) {
        this.verbose.debug(`Cancelling finding as user is already loaded.`);
        return;
      }

      this.verbose.info(`Storing current user before loading new user`);
      this.storeUser();
    }

    // try to find it in our filters first.
    let id = this.id(identifier);
    this.verbose.debug(`Loaded id?: ${id}`);
    if (id != undefined) {
      this.current_user = id;
      return;
    }
    let name = this.names(identifier);
    this.verbose.debug(`Loaded name?: ${id}`);
    if (name != undefined) {
      this.current_user = name;
      return;
    }

    // generate the json to get the user.
    let json = { name: identifier };
    let nan = Number(identifier);
    if (!Number.isNaN(nan)) {
      json.id = nan;
      json.name = undefined;
    }

    this.verbose.debug(`Attempting to load ${JSON.stringify(json)} from database`);
    // and load the user. Even if it doesn't exist.
    let user = await this.db.users.get(json);
    this.verbose.debug(`Found: `, user);
    if (user == undefined && json.name != undefined) {
      this.verbose.debug(`Attempting to search past names of data`);
      user = await this.db.users.where('past').anyOf(json.name).toArray();
      this.verbose.debug(`Found: `, user);
      user = user.length != 0 ? user[0] : undefined;
    }

    let userClass = await this.#userClass.create(user ?? identifier, this.db);
    this.verbose.info(`First user result is: `, userClass);
    if (typeof userClass === "number") {
      this.verbose.debug(`Making new user from previously found data`);
      userClass = new this.#userClass(await this.db.users.get({ id: userClass }));
    }
    this.verbose.debug(userClass);
    this.verbose.debug(userClass == null);
    this.verbose.debug(typeof userClass == "number");
    this.verbose.debug(userClass instanceof this.#userClass);
    if (userClass == null || typeof userClass == "number" || !(userClass instanceof this.#userClass)) {
      this.verbose.warn(`Cancelling load of user due to internal error.`)
      return;
    }

    this.verbose.info(`Storing and setting user! Loading completed!`);
    this.current_user = userClass;
    this.addItem(this.current_user);
  }

  async storeUser() {
    await this.db.users.add(this.current_user.database);
  }

  /** @type {User} */
  #userClass;
  get userClass() {
    return this.#userClass ?? User;
  }
  set userClass(v) {
    this.#userClass = v;
  }

  constructor(database) {
    super();
    this.addFilter('names', user => [user.name, ...user.past]);
    this.addFilter('id', user => user.id);
    this.verbose = new Verbose("UserManager", '#afe9ca');
    this.db = database;

    // store user upon leaving the page. Hence we don't lose any data.
    addEventListener('unload', () => this.storeUser());
  }
}
