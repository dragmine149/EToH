// EToH/ETOH/tests/BadgeManager.test.js
const { Badge, BadgeManager } = require('../newJs/BadgeManager.js');
const { GenericManager } = require('../newJs/DataManager.js');

describe('Badge', () => {
  test('should create a Badge instance with correct properties', () => {
    const badge = new Badge('Test Badge', [123, 456]);

    expect(badge.name).toBe('Test Badge');
    expect(badge.ids).toEqual([123, 456]);
    expect(badge.link).toBe('https://www.roblox.com/badges/123');
  });

  test('should not allow modification of properties added via __addProperty', () => {
    const badge = new Badge('Test Badge', [123, 456]);
    badge.name = 'New Name';

    expect(badge.name).toBe('Test Badge');
  });
});

describe('BadgeManager', () => {
  let badgeManager;

  beforeEach(() => {
    badgeManager = new BadgeManager();
  });

  test('should add a Badge instance to the manager', () => {
    const badge = new Badge('Test Badge', [123]);
    badgeManager.addBadge(badge);

    expect(badgeManager.names('Test Badge')).toContain(badge);
  });

  test('should filter uncompleted badges', () => {
    const badge = new Badge('Test Badge', [123]);
    badgeManager.addBadge(badge);

    const uncompleted = badgeManager.uncompleted([456]);
    expect(uncompleted).toEqual([]);
  });
});
