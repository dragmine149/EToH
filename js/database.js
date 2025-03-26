/**
 * A wrapper class for working with IndexedDB databases
 */
class Database {
  /**
   * Creates a new Database instance
   * @param {string} name - The name of the database
   * @param {number} version - The version number of the database
   * @param {function} upgradeFunction(db: IDBDatabase, oldVersion: number, newVersion: number): void | Promise<void> - Function called when database needs upgrading
   */
  constructor(name, version, upgradeFunction) {
    this.name = name;
    this.version = version;
    this.upgradeFunction = upgradeFunction;

    this.database = indexedDB.open(name, version);
    this.database.onerror = (event) => {
      console.error(`Error opening database ${name}: ${event.target.error}`);
    };
    this.database.onupgradeneeded = (event) => {
      this.upgradeFunction(event.target.result, event.oldVersion, event.newVersion);
    };
    this.database.onsuccess = (event) => {
      this.database = event.target.result;
    };
  }
}
