/**
 * Manages database updates by recording and executing store operations
 */
class DatabaseUpdater {
  /** @type {Array<Object>} List of store operations to perform */
  actions = [];

  /**
   * Creates a new DatabaseUpdater
   * @param {number} desiredVersion The desired database version
   */
  constructor(desiredVersion) {
    this.desiredVersion = desiredVersion;
  }

  /**
    * @typedef {Object} DatabaseIndex
    * @property {string} name The name of the index
    * @property {string} keyPath The keyPath for the index
    * @property {IDBIndexParameters} options Options for the index
    */

  /**
  * Records an action to create a new object store
  */
  createStore() {
    const self = this;
    return {
      /**
       * Creates a store with an auto-incrementing key
       * @param {string} storeName Name of the store to create
       * @param {string} keyname Property to use as key path (defaults to empty string)
       * @param {DatabaseIndex[]} indexes Array of indexes to create on the store
       * @param {Function} oncomplete Callback function when store creation completes
       */
      autokey(storeName, keyname = "", indexes = [], oncomplete = () => { }) {
        self.actions.push({
          type: 'createStore',
          storeName,
          options: {
            keyPath: keyname,
            autoIncrement: true,
          },
          indexes,
          oncomplete
        });
      },

      /**
       * Creates a store with a specified key path
       * @param {string} storeName Name of the store to create
       * @param {string} keyPath Property to use as key path
       * @param {DatabaseIndex[]} indexes Array of indexes to create on the store
       * @param {Function} oncomplete Callback function when store creation completes
       */
      key(storeName, keyPath, indexes = [], oncomplete = () => { }) {
        self.actions.push({
          type: 'createStore',
          storeName,
          options: {
            keyPath,
          },
          indexes,
          oncomplete
        });
      }
    };
  }


  /**
   * Records an action to delete an object store
   * @param {string} storeName Name of the store to delete
   */
  deleteStore(storeName) {
    this.actions.push({
      type: 'deleteStore',
      storeName
    });
  }

  /**
   * Records an action to modify an existing object store
   * @param {string} storeName Name of the store to change
   * @param {string} keyPath New key path for the store
   * @param {Object} options New store options
   */
  changeStore(storeName, keyPath, options) {
    this.actions.push({
      type: 'changeStore',
      storeName,
      options: {
        keyPath,
        ...options
      }
    });
  }

  /**
   * Update a database according to the rules already provided.
   * @param {IDBDatabase} db The database to update
   */
  updateDatabase(db) {
    for (let action of this.actions) {
      switch (action.type) {
        case 'createStore':
          let objectStore = db.createObjectStore(action.storeName, action.options);
          for (let index of action.indexes) {
            objectStore.createIndex(index.name, index.keyPath, index.options);
          }
          objectStore.transaction.oncomplete = (event) => {
            console.log(`Object store ${action.storeName} created`);
            action.oncomplete(db, event);
          };
          break;

        case 'deleteStore':
          db.deleteObjectStore(action.storeName);
          break;

        case 'changeStore':
          console.warn('changeStore not implemented');
          // db.deleteObjectStore(action.storeName);
          // db.createObjectStore(action.storeName, action.options);
          break;
      }
    }
  }
}
class UpdateDatabase {
  /**
   * Class to manage database updates with delayed execution.
   */
  constructor() {
    this.pendingStores = [];
    this.currentStore = null;
  }

  /**
   * @typedef {Object} StoreConfiguration
   * @property {(name: string) => StoreConfiguration} setName Sets the name of the store
   * @property {(keyPath: string) => StoreConfiguration} setKeyPath Sets the key path for the store
   * @property {(enable?: boolean) => StoreConfiguration} setAutoIncrement Enables auto increment for the store
   * @property {() => IndexConfiguration} createIndex Starts configuring a new index for the store
   */

  /**
   * @typedef {Object} IndexConfiguration
   * @property {(name: string) => IndexConfiguration} setName Sets the name of the index
   * @property {(property: string) => IndexConfiguration} setProperty Sets the property/keyPath for the index
   * @property {(isUnique?: boolean) => IndexConfiguration} setUnique Sets the index as unique
   * @property {(isMultiEntry?: boolean) => IndexConfiguration} setMultiEntry Sets the index as multi-entry
   * @property {() => StoreConfiguration} flush Finishes configuring this index
   */

  /**
   * Creates a store configuration and returns a chainable API
   * @returns {StoreConfiguration} A chainable store configuration object
   */
  createStore() {
    let currentStore = {
      __name: '',
      __keyPath: '',
      __autoIncrement: false,
      __indexes: [],

      /**
       * Sets the name of the store
       * @param {string} name The name of the store
       * @returns {StoreConfiguration} The chainable store configuration object
       */
      setName: (name) => {
        currentStore.__name = name;
        return currentStore;
      },

      /**
       * Sets the key for the store, aka the name. This is recommended to be set. Must be unique.
       * @param {string} keyPath The key to use.
       * @returns {StoreConfiguration} The chainable store configuration object
       */
      setKeyPath: (keyPath) => {
        currentStore.__keyPath = keyPath;
        return currentStore;
      },

      /**
       * Enables auto increment for the store. Doesn't allow unique key values and generates the keys automatically.
       * @param {boolean} enable Whether to enable auto increment (defaults to true)
       * @returns {StoreConfiguration} The chainable store configuration object
       */
      setAutoIncrement: (enable = true) => {
        currentStore.__autoIncrement = enable;
        return currentStore;
      },

      /**
       * Starts configuring a new index for the store
       * @returns {IndexConfiguration} A chainable index configuration object
       */
      createIndex: () => {
        let currentIndex = {
          __name: null,
          __keyPath: null,
          __unique: false,
          __multiEntry: false,

          /**
           * Sets the name of the index. Allows one to get the data quicker as `.get` can be used instead of looping.
           * @param {string} name The name of the index
           * @returns {IndexConfiguration} The chainable index configuration object
           */
          setName: (name) => {
            currentIndex.__name = name;
            return currentIndex;
          },

          /**
           * Sets the property/keyPath for the index
           * @param {string[]} property The property to index, Can be multiple properties for a "compound key"
           * @returns {IndexConfiguration} The chainable index configuration object
           */
          setProperty: (property) => {
            currentIndex.__keyPath = property;
            return currentIndex;
          },

          /**
           * Sets the index as unique. Doing so, will force this entry to by unique.
           * @param {boolean} isUnique Whether the index is unique (defaults to true)
           * @returns {IndexConfiguration} The chainable index configuration object
           */
          setUnique: (isUnique = true) => {
            currentIndex.__unique = isUnique;
            return currentIndex;
          },

          /**
           * Sets the index as multi-entry. Works better with the property being an array.
           * Setting to false will force all entries in the `property` to exists, setting to true requires ANY of the properties to exist.
           * If set along side unique, requires ALL entries in the key to be unique. Can be useful for stuff like table relationships.
           * @param {boolean} isMultiEntry Whether the index is multi-entry (defaults to true)
           * @returns {IndexConfiguration} The chainable index configuration object
           */
          setMultiEntry: (isMultiEntry = true) => {
            currentIndex.__multiEntry = isMultiEntry;
            return currentIndex;
          },

          /**
           * Finishes configuring this index and returns to store configuration
           * @returns {StoreConfiguration} The chainable store configuration object
           */
          flush: () => {
            currentStore.__indexes.push(currentIndex);
            currentIndex = null;
            return currentStore;
          }
        };

        return currentIndex;
      }
    };

    return currentStore;
  }
}

let ud = new UpdateDatabase();
ud.createStore()
  .setName('users')
  .setKeyPath('id')
  .setAutoIncrement(true)

  .createIndex()
  .setName('email')
  .setProperty('email')
  .setUnique(true)
  .flush()

  .createIndex()
  .setName('name')
  .setProperty('name')
  .flush()



/**
 * A wrapper class for working with IndexedDB databases
 */
class Database {
  /** @type {string} */
  name;

  /** @type {number} */
  version;

  /** @type {DatabaseUpdater[]} */
  __updaters;

  /** @type {IDBDatabase} */
  database;

  /**
   * Creates a new Database instance
   * @param {string} name - The name of the database
   * @param {number} version - The version number of the database
   * @param {DatabaseUpdater[]} updaters - list of classes of what to pass the database through when upgrading,
   */
  constructor(name, version, updaters) {
    this.name = name;
    this.version = version;
    updaters.sort((a, b) => a.version - b.version);
    this.__updaters = updaters;
    this.open();
  }

  get __databaseReady() {
    return this.database === undefined ? false : true;
  }

  close() {
    this.database.close();
  }

  open() {
    let databaseRequest = indexedDB.open(this.name, this.version);
    databaseRequest.onerror = (event) => {
      console.error(`Error opening database ${this.name}: ${event.target.error}`);
    };
    databaseRequest.onupgradeneeded = (event) => {
      let oldVersion = event.oldVersion;
      if (this.__updaters.length === 0) {
        console.error(`No updaters provided for database ${this.name}. This might require a database reset to fix! (Trying to update from ${oldVersion} to ${this.version})`);
        return;
      }

      /** @type {IDBDatabase} */
      let db = event.target.result;
      for (let updater of this.__updaters) {
        if (updater.desiredVersion == oldVersion + 1) {
          updater.updateDatabase(db);
          oldVersion = updater.desiredVersion;
        }
      }
    };
    databaseRequest.onsuccess = (event) => {
      this.database = event.target.result;
      this.database.onerror = this.errorFunction;
    };
  }


  store_data(store, data, oncomplete = (event) => { }, onerror = (event) => { }, onabort = (event) => { }) {
    let transaction = this.database.transaction(store, "readwrite");

    transaction.oncomplete = (event) => {
      console.log("Transaction completed");
      console.log(event);
    }
    transaction.onabort = (event) => {
      console.error(`Transaction aborted: ${event.target.error}`);
      onabort(event);
    }
    transaction.onerror = (event) => {
      console.error(`Transaction error: ${event.target.error}`);
      onerror(event);
    }

    let objectStore = transaction.objectStore(store);

    if (Array.isArray(data)) {
      for (let item of data) {
        objectStore.add(item);
      }
      return;
    }

    objectStore.add(data);
  }

  delete_data(store, query, oncomplete = (event) => { }, onerror = (event) => { }, onabort = (event) => { }) {
    let request = this.database
      .transaction(store, "readwrite")
      .objectStore(store)
      .delete(query);

    request.onsuccess = (event) => {
      console.log(`Deleted data from ${store}`);
      oncomplete(event);
    };
    request.onerror = (event) => {
      console.error(`Error deleting data from ${store}: ${event.target.error}`);
      onerror(event);
    };
    request.onabort = (event) => {
      console.error(`Transaction aborted: ${event.target.error}`);
      onabort(event);
    };
  }

  get_data(store, query, oncomplete = (event) => { }, onerror = (event) => { }, onabort = (event) => { }) {
    let request = this.database
      .transaction(store)
      .objectStore(store)
      .get(query);

    request.onsuccess = (event) => {
      console.log(`Retrieved data from ${store}`);
      oncomplete(event);
      console.log(event.target.result);
      return event.target.result;
    };
    request.onerror = (event) => {
      console.error(`Error retrieving data from ${store}: ${event.target.error}`);
      onerror(event);
    };
    request.onabort = (event) => {
      console.error(`Transaction aborted: ${event.target.error}`);
      onabort(event);
    };
    return request;
  }
}


let dbu1 = new DatabaseUpdater(1);
dbu1.createStore().key("Users", "id", [
  {
    name: "name",
    keyPath: "name",
    options: {
      multiEntry: false,
      unique: false
    }
  }
]);


let updaters = [
  dbu1
];
class TestDatabase {
  constructor() {
    this.database = new Database('test', 1, updaters);
  }
}

let testDatabase = new TestDatabase();
let data = [
  {
    "id": 6057,
    "name": "Test",
    "towers": {
      "Tower of Annoyingly Simple Trials": dayjs()
    }
  },
  {
    "id": 12,
    "name": "Test2",
    "towers": {
      "Tower of Annoyingly Simple Trials": null
    }
  }
]

// How IndexedDB works:
//
// - Create the database(s).
// - Create stores inside of the databases upon upgrading the Database
// - Store data inside those stores.
//
// Stores acts like tables, Upon storing data, the keyPath is required unless autogen. `{id: 'e'}` Indexes are not required, any other field can be added however.
// If we want to create new store/user. Then we need to close and reopen per user. (seems expensive)
//
// Returning data happens via a `onsuccess` function, with event.target.result as the data. This makes it harder to just "Return" said data.
