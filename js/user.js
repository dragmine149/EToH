class User {
  /** @type {number} */
  id;

  /** @type {string[]} */
  past = [];

  /** @type {string} */
  username;

  /** @type {string} */
  display;

  /** @type {boolean} */
  played = false;

  /** @type {number} */
  // If we run out of space, we'll remove the users that have not been viewed in a while first.
  last_viewed = 0;

  get ui_name() {
    if (this.display != null) {
      return `${this.display} (@${this.username})`;
    }
    return this.username
  }

  get link() {
    return `https://www.roblox.com/users/${this.id}/profile`;
  }

  get database() {
    return {
      id: this.id,
      past: this.past,
      username: this.username,
      display: this.display,
      played: this.played,
      viewed: this.last_viewed
    };
  }

  set database(data) {
    this.id = data.id;
    this.past = data.past;
    this.username = data.name;
    this.display_name = data.display;
    this.played = data.played;
    this.viewed = new Date().getTime();
  }

  /**
  * Loads a new user element
  * @param {{id: number, name: string, display: string, past: string[], played: boolean}} user_information Information to identify this user
  */
  constructor(user_information) {
    this.database = user_information;
    this.verbose = new Verbose("user", "#182409");
  }

  async updateFromServer() {
    /**
    * Gets the basic user information from the server if we don't already have it.
    * @param {number} id
    * @param {string} name
    */
    async function __getBasic(id, name) {
      // don't need to worry about display name as thats eh.
      if (Number.isInteger(id) && (name != null && name != '')) {
        // we already have the basics.
        return true;
      }

      let networkUserRequest = await tryCatch(network.retryTilResult(new Request(
        `${CLOUD_URL}/users/${(id ?? name)}`
      )));
      ui.updateLoadingStatus(`Trying to convert data to json. (data received)`);

      if (networkUserRequest.error) {
        ui.showError(`Failed to contact cloud for information about user!`);
        return;
      }

      let networkUserJSON = await tryCatch(networkUserRequest.data.json());

      if (networkUserJSON.error) {
        ui.showError(`Failed to convert the returned data from the cloud to json!`);
        return;
      }
    }


  }
}

class UserManager {
  /** @type {User[]} */
  previousLoaded = [];

  /**
  * Gets the user information object from the database
  * @param {string|number} user Username/user id/past name. A way to get the user.
  * @private
  * @returns The user information object as stored in the database
  */
  async __userDatabase(user) {
    if (typeof user == 'string') {
      let userDB = await etohDB.users.get({ name: user });
      if (userDB) {
        return userDB;
      }

      return await etohDB.users.where('past').anyOf(user).toArray();
    }

    return await etohDB.users.get({ id: user });
  }

  async updateUserFromCloud(user) {
    ui.hideError();
    ui.updateLoadingStatus(`Updating user information from the cloud!`)
    let networkUserRequest = await tryCatch(network.retryTilResult(new Request(
      `${CLOUD_URL}/users/${user}`
    )));
    ui.updateLoadingStatus(`Trying to convert data to json. (data received)`);

    if (networkUserRequest.error) {
      ui.showError(`Failed to contact cloud for information about user!`);
      return;
    }

    let networkUserJSON = await tryCatch(networkUserRequest.data.json());

    if (networkUserJSON.error) {
      ui.showError(`Failed to convert the returned data from the cloud to json!`);
      return;
    }

    ui.updateLoadingStatus(`Converted to json, updating past names`)
    /** @type {{id: number, name: string, display: string, past_name: string[], played: boolean}} */
    let networkUser = networkUserJSON.data;
    if (typeof user == 'string' && networkUser.name != user) {
      networkUser.past_name.push(user);
    }

    ui.updateLoadingStatus(`Checking if user has played before`);

  }

  /**
  * Find a roblox user in local storage or does the representative network request.
  * @param {string|number} user Information to identify the user.
  * @returns {Promise<{id: number, name: string, display: string, past_name: string[], played: boolean}>} The user data
  */
  async findUser(user) {
    // attempts to see if we have user data stored locally.
    ui.updateLoadingStatus(`Attempting to find user in database: `, userDb, true)
    let userDb = this.__userDatabase(user);
    if (userDb) {
      return userDb;
    }

    let played = getBadgeFromCaegories(['other', 'Playing'], 'Played');
    if (played == null || played.length < 1) {
      networkUser.played = false;
      return networkUser;
    }
    let badge = played[0];
    networkUser.played = network.getEarlierBadge(networkUser.id, badge.primaryId, badge.ids[0]);
    return networkUser;
  }

  async getUser(user_information) {
    let previous = this.previousLoaded.filter(prev => prev.id == user_information.id);
    if (previous.length > 0) {
      return previous[0]
    }

    let user = new User(user_information);
    this.previousLoaded.push(user);
    return user;
  }
}

let users = new UserManager();
