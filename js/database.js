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
   * Records an action to create a new object store
   * @param {string} storeName Name of the store to create
   * @param {string} keyPath Key path for the store
   * @param {Object} options Additional store options
   * @param {string} index Name of the index to create
   * @param {Function} oncomplete Callback function to execute when the operation is complete
   */
  createStore(storeName, keyPath, options, index, oncomplete) {
    this.actions.push({
      type: 'createStore',
      storeName,
      options: {
        keyPath,
        ...options
      },
      index,
      oncomplete
    });
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
          for (let index of action.index) {
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

/**
 * A wrapper class for working with IndexedDB databases
 */
class Database {
  /** @type {function} */
  errorFunction;
  /** @type {function} */
  successFunction;

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
    return this.database ? true : false;
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


let dbu1 = new DatabaseUpdater(1);
dbu1.createStore("users", "id", null, [
  {
    "name": "name",
    "keyPath": "name",
    "options": {
      "unique": true
    }
  }
], (db, event) => {
  // console.log(`Store created: ${event.target.result.name}`);

  db.transaction("users", "readwrite").objectStore("users").add(data[0]);
  db.transaction("users", "readwrite").objectStore("users").add(data[1]);
});

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
