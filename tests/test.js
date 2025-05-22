class TestTemplate {
  /** @type {(() => {})[]} List of functions to call before the test. */
  before;

  /** @type {Object.<string, Expect[]>} The tests to run. */
  functions;

  constructor() {
    this.before = [];
    this.functions = {};
  }
}

const TESTTYPE = Object.freeze({
  INFO: 0,
  DEBUG: 1,
  WARNING: 2,
  ERROR: 3,
  SUCCESS: 4,
  FAILED: 5
})

class Test {
  /** @type {Object.<string, TestTemplate>}} Information about every single test */
  test_data = {}
  /** The current test we are working on. */
  current = {
    category: "",
    test: ""
  }

  log(type, message) {
    let icon;
    let consoleMethod;

    switch (type) {
      case TESTTYPE.INFO:
        icon = 'ℹ️'; // Information icon
        consoleMethod = console.info;
        break;
      case TESTTYPE.DEBUG:
        icon = '🐞'; // Ladybug for debugging
        consoleMethod = console.log; // console.debug isn't universally styled
        break;
      case TESTTYPE.WARNING:
        icon = '⚠️'; // Warning sign
        consoleMethod = console.warn;
        break;
      case TESTTYPE.ERROR:
        icon = '❌'; // Red cross for error
        consoleMethod = console.error;
        break;
      case TESTTYPE.SUCCESS: // Added case for SUCCESS
        icon = '✅'; // Checkmark for success
        consoleMethod = console.log; // Often success can just be a regular log
        break;
      case TESTTYPE.FAILED: // Added case for FAILED
        icon = '❌'; // Using the same red cross as ERROR for a clear indication of failure
        consoleMethod = console.error; // Or you could use console.warn depending on severity
        break;
      default:
        icon = '❓'; // Question mark for unknown type
        consoleMethod = console.log;
    }

    consoleMethod(`${icon} ${message}`);
  }

  /**
  * Runs this code before every test.
  * @param {() => {}} setup The code to run. Must return a json of globals to use in the test.
  */
  before(setup) {
    this.test_data[this.current.category].before.push(setup);
  }

  /**
  * Describe a new category of tests.
  * @param {String} category_name The name of the category.
  * @param {() => any)} test_function The function for all the tests.
  */
  describe(category_name, test_function) {
    this.current.category = category_name;
    this.test_data[category_name] = new TestTemplate();
    test_function();
    this.current.category = "";
  }

  /**
  * Describe a test and what to do.
  * @param {String} test_name The name of the test.
  * @param {() => any)} test_function The function to test.
  */
  test(test_name, test_function) {
    this.current.test = test_name;
    this.test_data[this.current.category].functions[this.current.test] = [];
    test_function();
    this.current.test = "";
  }

  expect() {
    let expect = new Expect(name);
    this.test_data[this.current.category].functions[this.current.test].push(expect);
    return expect;
  }

  /**
  * Executes a single test.
  * @param {string} test_name
  */
  execute_test(test_name, index, count) {
    this.log(TESTTYPE.INFO, `Starting test for: ${test_name} (${index}/${count})`);
    let info = this.test_data[test_name];
    let globals = info.before.flatMap((v) => v())
    let combinedGlobals = {};
    globals.forEach(global => {
      Object.assign(combinedGlobals, global);
    });;
    let length = Object.keys(info.functions).length;
    let passed = TESTTYPE.SUCCESS;
    let msg = "";

    Object.entries(info.functions).forEach((func, index) => {
      this.log(TESTTYPE.INFO, `Test ${func[0]}: ${index + 1}/${length}`);

      let sub_length = func[1].length;
      func[1].forEach((func, sub_index) => {
        this.log(TESTTYPE.INFO, `Sub test: ${sub_index + 1}/${sub_length}`);
        let test_results = func.execute(combinedGlobals);
        this.log(test_results[0] ? TESTTYPE.SUCCESS : TESTTYPE.FAILED, `Test id: ${index + 1} resulted in a ${test_results[0] ? 'pass' : 'fail'}`);
        passed = test_results[0] ? passed : TESTTYPE.FAILED;
        msg += test_results[0] ? "" : test_results[1];
      });

      this.log(TESTTYPE.INFO, `End of ${index + 1}`);
    });

    this.log(passed, `Results: ${passed == TESTTYPE.SUCCESS ? 'passed' : 'failed'}\n${msg}`);
  }

  executeAll() {
    this.log(TESTTYPE.INFO, `Starting all tests...`);
    let categories = Object.keys(this.test_data).length;
    Object.keys(this.test_data).forEach((test, index) => {
      this.log(TESTTYPE.INFO, `Starting testing category: ${test}`);
      this.execute_test(test, index + 1, categories);
      console.log('---------------------------------------------------------------------------------------------------------');
    })
    this.log(TESTTYPE.INFO, `All tests finished`);
  }
}

class Expect {
  /** @type {Object.<string, {
    info: any, hit: bool, condition: () => [bool, string], reason: string
  }>} */
  exptects = {};

  /** @type {() => any} */
  test;

  #exptects(expect, condition, info) {
    if (this.exptects[expect]) {
      console.warn(`Attempted to add another ${expect} to an expect where a ${expect} already exists.`);
      return;
    }
    this.exptects[expect] = {
      info,
      condition: condition?.bind(this, info),
      hit: false,
      reason: "",
    }
    return this;
  }

  #newExpect(name, condition) {
    this[name] = this.#exptects.bind(this, name, condition);
    return this[name];
  }

  from(test_callback) {
    if (this.test) {
      console.warn(`Attempted to add another test to an expect where a test already exists.`);
      return;
    }
    this.test = test_callback;
    return this;
  }

  execute(globals) {
    try {
      let result = this.test(globals);

      Object.values(this.exptects).forEach((v) => {
        let r = v.condition(result);
        v.hit = r[0];
        v.reason = r[1];
      })
      console.log(result);
    }
    catch (e) {
      Object.values(this.exptects).forEach((v) => {
        let r = v.condition(e);
        v.hit = r[0];
        v.reason = r[1];
      })
      console.error(e);
    }

    let success = true;
    let msg = "";
    Object.entries(this.exptects).forEach(([key, info], index) => {
      console.log(index);
      success = info.hit ? success : false;
      if (!info.hit) msg += `${index} -> Condition ${key} was not met: ${info.reason}\n`;
    });

    return [success, msg];
  }

  constructor(name) {
    this.name = name;
    this.#newExpect('type', (v, t) => {
      if (t == undefined) return [false, `Found result of 'from' to be undefined.`];
      return v(t) ? [true, ''] : [false, `Expected type ${typeof (v)} found type ${typeof (t)}`]
    });
    this.#newExpect('error',
      /** @param {Error} v @param {Error} i  */
      (v, i) =>
        (v.name === i.name && v.message === i.message && v.stack !== i.stack)
          ? [true, '']
          : [false, `Expected error with name "${i.name}" and message "${i.message}", but got name "${v.name}" and message "${v.message}".`]
    );
  }
}

let test = new Test();




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
});

test.executeAll();
