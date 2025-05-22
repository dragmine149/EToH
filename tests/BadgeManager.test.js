/**
* @typedef {import('./test')}
* @typedef {import('../newJs/BadgeManager')}
*/

test.describe("BadgeManager", () => {
  test.before(() => {
    return {
      badgeManager: new BadgeManager(),
    }
  });

  test.test("cannot add anything else that is not a badge", () => {
    test.expect()
      .error(new Error("Only instances of Badge can be added to BadgeManager."))
      .from((global) => {
        global.badgeManager.addBadge(24);
      });
  });
});

test.describe("Badge", () => {
  test.before(() => {
    return {
      badge: new Badge("test", Math.floor(Math.random() * 1000000))
    }
  })

  test.test("Creating a new badge", () => {
    test.expect()
      .type((v) => v instanceof Badge)
      .from(() => {
        return new Badge("test", 537920)
      })
  })
})

// test.executeAll();
