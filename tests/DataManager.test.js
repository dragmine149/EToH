const { GenericManager } = require('../newJs/DataManager');

describe('GenericManager', () => {
  let manager;

  beforeEach(() => {
    manager = new GenericManager();
  });

  test('should add an item and retrieve it via a filter', () => {
    manager.addFilter('name', item => item.name);

    const item = { name: 'Test Item' };
    manager.addItem(item);

    expect(manager.name('Test Item')).toContain(item);
  });

  test('should return undefined for non-existent filter keys', () => {
    manager.addFilter('name', item => item.name);

    expect(manager.name('Non-existent')).toBeUndefined();
  });
});
