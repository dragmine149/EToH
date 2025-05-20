const { User, UserManager } = require('../newJs/user');

describe('User', () => {
  test('should create a User instance with correct properties', () => {
    const user = new User({ id: 1, name: 'TestUser', display: 'Test Display' });

    expect(user.id).toBe(1);
    expect(user.name).toBe('TestUser');
    expect(user.display).toBe('Test Display');
  });
});

describe('UserManager', () => {
  let userManager;

  beforeEach(() => {
    userManager = new UserManager();
  });

  test('should add a user and retrieve it by name', () => {
    const user = new User({ id: 1, name: 'TestUser' });
    userManager.addItem(user);

    expect(userManager.names('TestUser')).toContain(user);
  });
});
