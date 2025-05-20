const { TowerManager } = require('../newJs/Towers');

describe('TowerManager', () => {
  let towerManager;

  beforeEach(() => {
    towerManager = new TowerManager();
  });

  test('should return the correct difficulty word', () => {
    expect(towerManager.getDifficultyWord(1)).toBe('Easy');
    expect(towerManager.getDifficultyWord(11)).toBeUndefined();
  });

  test('should return the correct difficulty description', () => {
    expect(towerManager.getDifficulty(1.45)).toBe('Mid Easy');
  });
});
