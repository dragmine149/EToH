/*global test, BadgeManager, Badge*/
/**
* @typedef {import('./test')}
* @typedef {import('../newJs/BadgeManager')}
*/

test.describe("Badge and BadgeManager", () => {
  test.before(() => {
    let bm = new BadgeManager();
    bm.addBadge(new Badge("Test 1", 1));
    bm.addBadge(new Badge("Test 2", [2, 3]));
    bm.addBadge(new Badge("Test 3", 4));

    return {
      badge: new Badge("Test Badge", 42),
      badgeManager: bm,
    }
  });

  test.test("BadgeManager: cannot add anything else that is not a badge", /** @param {Expect} expect */
    (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
      (globals) => globals.badgeManager.addBadge("e")
    )
      .catch_throw(Error, "Only instances of Badge can be added to BadgeManager.")
  );

  test.test("Badge setup is correct", /** @param {Expect} expect */
    (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
      () => new Badge("Test Badge", 42))
      .exists_type("result", Badge)
      .exists_type("result.ids", Array)
      .exists_type("result.name", "string")
      .is("result.name", "Test Badge")
      .is("result.ids.[0]", 42)
  );

  test.test("Badge can not be modified after setup", /** @param {Expect} expect */
    (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
      (globals) => {
        let badge = globals.badge;
        badge.ids.push(56);
        badge.name = "Hello";
        return badge;
      })
      .exists_type("result", Badge)
      .exists_type("result.ids", Array)
      .exists_type("result.name", "string")
      .is("result.name", "Test Badge")
      .is("result.ids.[0]", 42)
      .len("result.ids", 1)
  );

  test.test("Link return is correct", /** @param {Expect} expect */
    (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
      (globals) => globals.badge.link)
      .exists_type("result", "string")
      .is("result", "https://www.roblox.com/badges/42")
  );

  test.test("Link return is correct with multiple ids", /** @param {Expect} expect */
    (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
      () => {
        let badge = new Badge("Test Badge", [42, 56]);
        return { badge, link: badge.link };
      })
      .len("result.badge.ids", 2)
      .exists_type("result.link", "string")
      .is("result.link", "https://www.roblox.com/badges/42")
  );

  test.test("Can get multiple links", /** @param {Expect} expect */
    (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
      () => {
        let badge = new Badge("Test Badge", [42, 56]);
        return { badge, links: badge.links };
      })
      .len("result.badge.ids", 2)
      .exists_type("result.links", Array)
      .len("result.links", 2)
      .is("result.links.[0]", "https://www.roblox.com/badges/42")
      .is("result.links.[1]", "https://www.roblox.com/badges/56")
  );

  // test.test("Badge manager: Uncompleted returns all badges if not completed", /** @param {Expect} expect */
  //   (expect) => expect.from(/** @param {{badgeManager: BadgeManager, badge: Badge}} globals */
  //     (globals) => globals.badgeManager.uncompleted())
  //     .len("result", 3)
  // );

});
