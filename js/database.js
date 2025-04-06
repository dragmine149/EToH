class UpdateDatabase {
  /** @type {StoreConfiguration[]} */
  newStores = [];
  /** @type {string[]} */
  deletedStores = [];
  /** @type {Object.<string, { 'config': StoreConfiguration, 'map': (data: any) => any}>} */
  updatedStores = {};

  constructor(version) {
    this.version = version;

    console.log(`Initialising UpdateDatabase (v${this.version})`);
  }

  /**
   * @typedef {Object} StoreConfiguration
   * @property {(name: string) => StoreConfiguration} setName Sets the name of the store
   * @property {(keyPath: string) => StoreConfiguration} setKeyPath Sets the key path for the store
   * @property {(enable?: boolean) => StoreConfiguration} setAutoIncrement Enables auto increment for the store
   * @property {() => IndexConfiguration} createIndex Starts configuring a new index for the store
   * @property {(db: IDBDatabase) => void} __insert Internal property, not for use outside of this class. Inserts the store into the passed database.
   */

  /**
   * @typedef {Object} IndexConfiguration
   * @property {(name: string) => IndexConfiguration} setName Sets the name of the index
   * @property {(property: string) => IndexConfiguration} setProperty Sets the property/keyPath for the index
   * @property {(isUnique?: boolean) => IndexConfiguration} setUnique Sets the index as unique
   * @property {(isMultiEntry?: boolean) => IndexConfiguration} setMultiEntry Sets the index as multi-entry
   * @property {() => IndexConfiguration} createIndex Calls StoreConfiguration.createIndex.
   * @property {string} __parent Link to the parent StoreConfiguration, used to store data at the end of the array.
   */

  /**
   * Creates a store configuration and returns a chainable API
   * @returns {StoreConfiguration} A chainable store configuration object
   */
  createStore() {
    console.log(`creating store configuration`);
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
          __parent: currentStore,

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

          createIndex: () => {
            return currentStore.createIndex();
          }
        };

        currentStore.__indexes.push(currentIndex);
        return currentIndex;
      },

      /**
       * Inserts the store configuration into the database
       * @param {IDBDatabase} db The database to insert the store into
       * @throws {Error} If store name or keyPath is not set
       * @throws {Error} If an index name or keyPath is not set
       */
      __insert: (db) => {
        // store must have a name, no point if it doesn't.
        if (!currentStore.__name) {
          console.error('Store name must be set before inserting');
          return;
        }

        // According to devmo (https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Using_IndexedDB#creating_and_structuring_the_store) table structure, the keyPath and autoIncrement are optional.
        // add the store to the database.
        let store = db.createObjectStore(currentStore.__name, {
          keyPath: currentStore.__keyPath,
          autoIncrement: currentStore.__autoIncrement
        });

        for (const index of currentStore.__indexes) {
          // Indexes aren't usedful if they don't have at least a name and key path.
          let error = '';
          if (!index.__name) {
            error += 'Index name must be set before inserting';
          }
          if (!index.__keyPath) {
            error += error ? '. ' : '';
            error += 'Index keyPath must be set before inserting';
          }

          if (error) {
            console.error(error);
            // however, we continue as one broken is not all broken.
            continue;
          }

          store.createIndex(index.__name, {
            keyPath: index.__keyPath,
            unique: index.__unique
          });
        }
      }
    };

    this.newStores.push(currentStore);

    return currentStore;
  }

  /**
  * Deletes a store, can also be used to undo a createStore action.
  * @param {string} storeName The name of the store
  */
  deleteStore(storeName) {
    console.log(`creating delete store ${storeName} request`);
    this.deletedStores.push(storeName);
    this.newStores = this.newStores.filter(store => store.name !== storeName);
  }

  /**
  * Update a store by deleting the old one and making a new one. WILL KEEP ALL DATA OF PREVIOUS STORE (if possible)
  * @param {string} storeName The name of the old Store
  * @param {(IndexConfiguration|StoreConfiguration)} newStoreConfig The new store details to replace the old Store
  * @param {(event: Event) => void} map The function to call on all the data to get it ready for the new format.
  */
  updateStore(storeName, newStoreConfig, map) {
    console.log(`creating update store ${storeName} request`);

    if (newStoreConfig.__parent) {
      newStoreConfig = newStoreConfig.__parent;
    }

    this.newStores = this.newStores.filter(store => store.name !== storeName);
    this.updatedStores[storeName] = { 'config': newStoreConfig, 'map': map };
  }

  /**
  * If we have stuff to upgrade, grab the data here.
  * @param {Event} event The `onsuccess` event provided upon opening the database.
  * @param {(event: Event) => void} successFunction The function to call upon a successful action happening.
  */
  preExecute(event, successFunction) {
    console.log('Attempting preExecute update function')
    if (this.updatedStores.length <= 0) {
      // if we have nothing to update, don't do anything.
      console.log('No updates requested, continuing...');
      return successFunction(event);
    }
    // assume we have already done the version check.

    // close the current database
    /** @type {IDBDatabase} */
    let db = event.target.result;
    db.close();

    // open the old database
    let oldDb = indexedDB.open(db.name, this.version - 1);

    // get all the data for the specified store out of the database.
    this.upgradeData = {};
    Object.keys(this.updatedStores).forEach(storeName => {
      /** @type {IDBTransaction} */
      let readTransaction = oldDb.transaction(storeName, 'readonly');
      if (readTransaction == null) {
        console.log(`Transaction for ${storeName} is null`);
        return;
      }

      this.upgradeData[storeName] = readTransaction.objectStore(storeName).getAll();
    });

    oldDb.close();
  }

  /**
  * Update the database from the old version to the new version.
  * @param {Event} event The Event
  * @param {(event: Event) => void} successFunction Function that gets called upon update a success
  */
  execute(event, successFunction) {
    console.log('Executing update function');

    /** @type {IDBDatabase} */
    let db = event.target.result;
    /** @type {IDBTransaction} */
    let transaction = event.target.transaction;

    Object.keys(this.updatedStores).forEach(storeName => {
      if (!db.objectStoreNames.contains(storeName)) {
        console.warn(`Store ${storeName} does not exist, treating an upgrade like an addition`);
        this.newStores.push(this.updatedStores[storeName].config);
        return;
      }

      db.deleteObjectStore(storeName);
      this.updatedStores[storeName].__insert(db);

      if (this.upgradeData?.storeName) {
        let store = transaction.objectStore(storeName);
        this.upgradeData[storeName].forEach(data => {
          store.add(data);
        });
      }
    });

    this.newStores.forEach(storeData => {
      if (db.objectStoreNames.contains(storeData.__name)) {
        console.warn(`Store ${storeName} already exists`);
        return;
      }

      storeData.__insert(db);
    });
    this.deletedStores.forEach(storeName => {
      if (!db.objectStoreNames.contains(storeName)) {
        console.warn(`Store ${storeName} does not exist`);
        return;
      }

      db.deleteObjectStore(storeName);
    });
  }
}

// let ud = new UpdateDatabase();
// ud.updateStore('Users', ud
//   .createStore()
//   .setName('users')
//   .setKeyPath('id')
//   .setAutoIncrement(true)

//   .createIndex()
//   .setName('email')
//   .setProperty('email')
//   .setUnique(true)

//   .createIndex()
//   .setName('name')
//   .setProperty('name')
// );

// ud.createStore()
//   .setName('achievements')
//   .setKeyPath('id')
//   .setAutoIncrement(true)

//   .createIndex()
//   .setName('userId')
//   .setProperty('userId');

/**
 * A wrapper class for working with IndexedDB databases
 */
class Database {
  /** @type {string} */
  name;

  /** @type {number} */
  version;

  /** @type {UpdateDatabase[]} */
  __updaters;

  /** @type {IDBDatabase} */
  database;

  /**
   * Creates a new Database instance
   * @param {string} name - The name of the database
   * @param {number} version - The version number of the database
   * @param {UpdateDatabase[]} updaters - list of classes of what to pass the database through when upgrading,
   */
  constructor(name, version, updaters) {
    this.name = name;
    this.version = version;
    updaters.sort((a, b) => a.version - b.version);
    this.__updaters = updaters;
    this.open();
  }

  checkForUpdate(event) {
    /** @type {IDBDatabase} */
    let db = event.target.result;
    let version = db.version;
    db.close();

    if (version < this.version) {
      this.updateDatabase();
    }
  }

  updateDatabase() {

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
}


exports.UpdateDatabase = UpdateDatabase;
exports.Database = Database;

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
