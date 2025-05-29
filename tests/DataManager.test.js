/*global test, GenericManager*/
/**
* @typedef {import('./test')}
* @typedef {import('../newJs/DataManager')}
*/

test.describe("Data manager", () => {
  test.before(() => {
    return {
      generic: new GenericManager()
    }
  });

  test.test("Adding a filter should create a new callback function", /** @param {Expect} expect*/
    (expect) => expect.from(/** @param {{generic: GenericManager}} globals */
      (globals) => globals.generic.addFilter("test", (v) => v.test))
      .exists_type("globals.generic.test", "function")
  );

  test.test("Custom filters should always be an array", /** @param {Expect} expect*/
    (expect) => expect.from(/** @param {{generic: GenericManager}} globals */
      (globals) => {
        globals.generic.addFilter("test", (v) => v.test);
        globals.generic.addItem({ test: 5 });
        return {
          none: globals.generic.test(),
          any1: globals.generic.test("hello"),
          number: globals.generic.test(5),
          obj: globals.generic.test({ e: "f" })
        }
      })
      // .debug("result")
      .exists_type("result.none", Array)
      .exists_type("result.any1", Array)
      .exists_type("result.number", Array)
      .exists_type("result.obj", Array)
  );

  test.test("Filters will not filter invalid arguments",  /** @param {Expect} expect*/
    (expect) => expect.from(/** @param {{generic: GenericManager}} globals */
      (globals) => {
        globals.generic.addFilter("test", (v) => v.test);
        globals.generic.addItem({ test: 5 });
        globals.generic.addItem({ test2: 0 });
        globals.generic.addItem({ test3: 7 });
        globals.generic.addItem({ test: 1 });
        globals.generic.addItem({ test: 1 });
        return globals.generic.test();
      })
      .exists_type("result", Array)
      .array_has("result", 1)
      .array_has("result", 5)
      .len("result", 2)
  );

  test.test("Filters will return all objects of that filter thingy.",  /** @param {Expect} expect*/
    (expect) => expect.from(/** @param {{generic: GenericManager}} globals */
      (globals) => {
        globals.generic.addFilter("test", (v) => v.test);
        globals.generic.addItem({ test: 5 });
        globals.generic.addItem({ test2: 0 });
        globals.generic.addItem({ test3: 7 });
        globals.generic.addItem({ test: 1 });
        globals.generic.addItem({ test: 1 });
        return globals.generic.test(1);
      })
      // .debug("globals")
      .exists_type("result", Array)
      .len("result", 2)
  );

  test.test("Add item will not add an invalid key to filter.",  /** @param {Expect} expect*/
    (expect) => expect.from(/** @param {{generic: GenericManager}} globals */
      (globals) => {
        globals.generic.addFilter("test", (v) => v.test);
        globals.generic.addItem({ test: 5 });
        globals.generic.addItem({ test: NaN });
        globals.generic.addItem({ test: null });
        globals.generic.addItem({ test: 10 });
        globals.generic.addItem({ test: 1 });
        globals.generic.addItem({ test: undefined });
        return globals.generic.test();
      })
      .exists_type("result", Array)
      .array_has("result", 10)
      .array_has("result", 5)
      .len("result", 3)
  );

  test.test("Database should not allow spaces in filter", /** @param {Expect} expect*/
    (expect) => expect.from(/** @param {{generic: GenericManager}} globals */
      (globals) => globals.generic.addFilter("Filter with spaces in it", (v) => v.test))
      // .debug("globals")
      .catch_throw(Error, "Filter should not contain spaces for code readability reasons!")
  );
})
