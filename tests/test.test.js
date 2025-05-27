/*global test, Test*/
/**
* @typedef { import('./test') }
*/


test.describe("Test System Validation", () => {
  test.before({
    test: new Test()
  });

  test.test("Describe test",
    /** @param {Expect} expect  */
    (expect) => expect
      .from(
        /** @param {{test: Test}} globals  */
        (globals) => globals.test.describe("Test in a test", () => { })
      )
      .exists_type('globals.test', Test)
      .exists_type('globals.test.test_data', Array)
  );

  test.test("Before test working",
    /** @param {Expect} expect  */
    (expect) => expect
      .from(
        /** @param {{test: Test}} globals  */
        (globals) => {
          globals.test.describe("Test in a test", () => {
            globals.test.before({
              test_func: (function (a, b) {
                return a * b;
              })
            })
          })

        })
      .exists_type('globals.test.test_data', Array)
      .exists_type('globals.test.test_data.[0]', "object")
  );

  test.test("Testing the test",
    /** @param {Expect} expect  */
    (expect) => expect.from(
      /** @param {{test: Test}} globals  */
      (globals) => {
        //--------------------------------------------
        globals.test.describe("Test in a test", () => {
          globals.test.before({
            test_func: (function (a, b) {
              return a * b;
            })
          })
        })
        globals.test.test("5 * 3 = 15",
          /** @param {Expect} expect */
          (expect) => expect.from(
            /** @param {{test_func: (a: number, b: number) => number}} inner_globals  */
            (inner_globals) => inner_globals.test_func(5, 3)
          )
            .exists_type('result', "number")
            .is('result', 15)
        )
        //--------------------------------------------
      })
    // .log_no_has("error")
  )
});




// test.describe("Test System Validation", () => {
//   test.before(() => {
//     return [{
//       mockFunction: (input) => {
//         if (typeof input !== 'number') {
//           throw new Error("Input must be a number");
//         }
//         return input * 2;
//       }
//     }];
//   });

//   test.test("Describe and Test Registration", () => {
//     // This test primarily checks if describe and test are correctly registering tests.
//     // The assertion is implicit: if no errors are thrown during registration, it's considered a pass.
//   });

//   test.test("Before Hook Execution and Global Scope", () => {
//     test.expect()
//       .from((globals) => globals.mockFunction(5))
//       .type((t) => typeof t === 'number');

//     test.expect()
//       .from((globals) => globals.mockFunction(5))
//       .type((t) => t === 10);
//   });

//   test.test("Expect Type Assertion - Success", () => {
//     test.expect()
//       .from(() => "hello")
//       .type((t) => typeof t === 'string');
//   });

//   test.test("Expect Type Assertion - Failure", () => {
//     test.expect()
//       .from(() => 123)
//       .type((t) => typeof t === 'string');
//   });

//   test.test("Expect Error Assertion - Success", () => {
//     test.expect()
//       .from(() => { throw new Error("Test error"); })
//       .error({ name: "Error", message: "Test error" });
//   });

//   test.test("Expect Error Assertion - Failure (Incorrect Message)", () => {
//     test.expect()
//       .from(() => { throw new Error("Test error"); })
//       .error({ name: "Error", message: "Different error message" });
//   });

//   test.test("Expect Error Assertion - Failure (Incorrect Error Type)", () => {
//     test.expect()
//       .from(() => { throw new TypeError("Test error"); })
//       .error({ name: "Error", message: "Test error" });
//   });
// })
