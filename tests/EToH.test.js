const { Tower, Other } = require('../newJs/EToH');

describe('Tower', () => {
  test('should create a Tower instance with correct properties', () => {
    const tower = new Tower('Test Tower', [123], 5, 'Test Area');

    expect(tower.name).toBe('Test Tower');
    expect(tower.difficulty).toBe(5);
    expect(tower.area).toBe('Test Area');
  });

  test('should generate a short name for the tower', () => {
    const tower = new Tower('Tower of Testing', [123], 5, 'Test Area');

    expect(tower.shortName).toBe('ToT');
  });
});

describe('Other', () => {
  test('should create an Other instance with correct properties', () => {
    const other = new Other('Test Badge', [123], 'Test Category');

    expect(other.name).toBe('Test Badge');
    expect(other.category).toBe('Test Category');
  });
});
