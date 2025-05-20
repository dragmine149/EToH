const { Area, AreaManager } = require('../newJs/AreaManager');

describe('Area', () => {
  test('should create an Area instance with correct properties', () => {
    const requirements = { difficulties: { easy: 1 }, points: 10 };
    const area = new Area('Test Area', 'Parent Area', requirements);

    expect(area.name).toBe('Test Area');
    expect(area.parent).toBe('Parent Area');
    expect(area.requirements).toEqual(requirements);
  });
});

describe('AreaManager', () => {
  let areaManager;

  beforeEach(() => {
    areaManager = new AreaManager();
  });

  test('should add an Area instance to the manager', () => {
    const area = new Area('Test Area', null, { difficulties: {}, points: 0 });
    areaManager.addArea(area);

    expect(areaManager.name('Test Area')).toContain(area);
  });

  test('should throw an error when adding a non-Area instance', () => {
    expect(() => areaManager.addArea({})).toThrow('Only instances of Area can be added to AreaManager.');
  });
});
