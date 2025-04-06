const { Database, UpdateDatabase } = require('../database.js');

describe('UpdateDatabase', () => {
  /** @type {UpdateDatabase} */
  let updateDb;

  beforeEach(() => {
    updateDb = new UpdateDatabase(1);
  });

  test('creates a store configuration correctly', () => {
    const store = updateDb.createStore()
      .setName('testStore')
      .setKeyPath('id')
      .setAutoIncrement(true);

    expect(store.__name).toBe('testStore');
    expect(store.__keyPath).toBe('id');
    expect(store.__autoIncrement).toBe(true);
    expect(store.__indexes).toEqual([]);
  });

  test('creates store with index correctly', () => {
    const store = updateDb.createStore()
      .setName('testStore')
      .createIndex()
      .setName('testIndex')
      .setProperty('testProp')
      .setUnique(true)
      .__parent;

    expect(store.__indexes).toHaveLength(1);
    expect(store.__indexes[0].__name).toBe('testIndex');
    expect(store.__indexes[0].__keyPath).toBe('testProp');
    expect(store.__indexes[0].__unique).toBe(true);
  });

  test('deletes store correctly', () => {
    updateDb.createStore().setName('store1');
    updateDb.createStore().setName('store2');

    updateDb.deleteStore('store1');

    expect(updateDb.deletedStores).toContain('store1');
    expect(updateDb.newStores).toHaveLength(1);
  });

  test('updates store correctly', () => {
    const newConfig = updateDb.createStore()
      .setName('updatedStore')
      .setKeyPath('newId');

    const mapFn = jest.fn();

    updateDb.updateStore('oldStore', newConfig, mapFn);

    expect(updateDb.updatedStores.oldStore).toBeDefined();
    expect(updateDb.updatedStores.oldStore.config).toBe(newConfig);
    expect(updateDb.updatedStores.oldStore.map).toBe(mapFn);
  });
});

describe('Database', () => {
  let db;
  const mockUpdaters = [
    new UpdateDatabase(1),
    new UpdateDatabase(2)
  ];

  beforeEach(() => {
    db = new Database('testDb', 2, mockUpdaters);
  });

  test('initializes with correct properties', () => {
    expect(db.name).toBe('testDb');
    expect(db.version).toBe(2);
    expect(db.__updaters).toEqual(mockUpdaters);
  });

  test('opens database connection', () => {
    expect(mockIndexedDB.open).toHaveBeenCalledWith('testDb', 2);
  });

  test('handles database upgrade correctly', () => {
    const mockEvent = {
      oldVersion: 1,
      target: {
        result: {
          createObjectStore: jest.fn(),
          objectStoreNames: {
            contains: jest.fn().mockReturnValue(false)
          }
        }
      }
    };

    db.onupgradeneeded(mockEvent);

    // Verify upgrade logic was executed
    expect(mockEvent.target.result.createObjectStore).toHaveBeenCalled();
  });
});
