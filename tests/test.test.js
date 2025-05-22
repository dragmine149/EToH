/*global test, Test*/
/**
* @typedef { import('./test') }
*/


test.describe("Test System Validation", () => {
  test.before(() => {
    return {
      test: new Test()
    }
  })

  test.test("Describe test",
    /** @param {Expect} expect  */
    (expect) => {
      expect
        .from(
          /** @param {{test: Test}} globals  */
          (globals) => globals.test.describe("Test in a test", () => { })
        )
        .global_exists(
          /** @param {{test: Test}} globals  */
          (globals) => Object.keys(globals.test.test_data).includes("Test in a test") ? [true, ''] : [false, 'Failed to find `Test in a Test` in `globals.test.test_data`']
        )
        .global_exists(
          /** @param {{test: Test}} globals  */
          (globals) => globals.test.current.category === "" ? [true, ''] : [false, '`globals.test.current.category` failed to reset']
        )
        .type(
          (v) => v instanceof Test
        )
    });

  // test.test("")


});




test.describe("Test System Validation", () => {
  test.before(() => {
    return [{
      mockFunction: (input) => {
        if (typeof input !== 'number') {
          throw new Error("Input must be a number");
        }
        return input * 2;
      }
    }];
  });

  test.test("Describe and Test Registration", () => {
    // This test primarily checks if describe and test are correctly registering tests.
    // The assertion is implicit: if no errors are thrown during registration, it's considered a pass.
  });

  test.test("Before Hook Execution and Global Scope", () => {
    test.expect()
      .from((globals) => globals.mockFunction(5))
      .type((t) => typeof t === 'number');

    test.expect()
      .from((globals) => globals.mockFunction(5))
      .type((t) => t === 10);
  });

  test.test("Expect Type Assertion - Success", () => {
    test.expect()
      .from(() => "hello")
      .type((t) => typeof t === 'string');
  });

  test.test("Expect Type Assertion - Failure", () => {
    test.expect()
      .from(() => 123)
      .type((t) => typeof t === 'string');
  });

  test.test("Expect Error Assertion - Success", () => {
    test.expect()
      .from(() => { throw new Error("Test error"); })
      .error({ name: "Error", message: "Test error" });
  });

  test.test("Expect Error Assertion - Failure (Incorrect Message)", () => {
    test.expect()
      .from(() => { throw new Error("Test error"); })
      .error({ name: "Error", message: "Different error message" });
  });

  test.test("Expect Error Assertion - Failure (Incorrect Error Type)", () => {
    test.expect()
      .from(() => { throw new TypeError("Test error"); })
      .error({ name: "Error", message: "Test error" });
  });
})
