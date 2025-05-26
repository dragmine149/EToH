const enabled_log = false;

const TESTTYPE = Object.freeze({
  INFO: 0,
  DEBUG: 1,
  WARNING: 2,
  ERROR: 3,
  SUCCESS: 4,
  FAILED: 5
})

class Test {
  test_data = [];

  log(type, prefix, ...params) {
    let consoleMethod;

    switch (type) {
      case TESTTYPE.INFO:
        consoleMethod = console.info;
        break;
      case TESTTYPE.DEBUG:
        consoleMethod = console.debug();
        break;
      case TESTTYPE.WARNING:
        consoleMethod = console.warn;
        break;
      case TESTTYPE.ERROR:
        consoleMethod = console.error;
        break;
      case TESTTYPE.SUCCESS:
        consoleMethod = console.log;
        break;
      case TESTTYPE.FAILED:
        consoleMethod = console.error;
        break;
      default:
        consoleMethod = console.log;
    }

    consoleMethod(...prefix, ...params);

    if (new URL(location).hostname == "localhost" && enabled_log) {
      fetch(`${location.origin}/log`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(params)
      }).catch(() => { });
    }

  }

  /**
  * Runs this code before every test.
  * @param {() => {}} setup The code to run. Must return a json of globals to use in the test.
  */
  before(setup) {
    this.test_data.push(setup);
    // we also add the logging function as a global. This allows tests to report more things.
    this.test_data.push({
      log: this.log
    })
  }

  /**
  * Describe a new category of tests.
  * @param {String} category_name The name of the category.
  * @param {() => any)} test_function The function for all the tests.
  * @returns {{exclude: () => any, include: () => any}} Functions that can further affect the test suite.
  */
  describe(category_name, test_function) {
    this.log(TESTTYPE.INFO, [`%cStarting test suite:%c`, `color: cyan`, ``], `${category_name}`);

    this.test_data = [];
    test_function();
  }

  /**
  * Describe a test and what to do.
  * @param {String} test_name The name of the test.
  * @param {() => any)} test_function The function to test.
  */
  test(test_name, test_function) {
    this.log(TESTTYPE.INFO, [`%cStarting test:%c`, `color: orange`, ``], `${test_name}`);

    let globals = this.test_data.map((v) => (typeof v == "function") ? v() : v)
    let combinedGlobals = {};
    globals.forEach(global => {
      Object.assign(combinedGlobals, global);
    });;

    let expect = new Expect(combinedGlobals);
    let result = test_function(expect);
    this.log(TESTTYPE.INFO, [`%cFinished test:%c`, `color: orange`, ``], `${test_name}`);
    this.log(result.state.passed ? TESTTYPE.SUCCESS : TESTTYPE.FAILED, [`%cTest results: %c`, `color: orange`, ``], result.state.passed ? `Passed!` : `Failed: ${result.state.reason}`);
  }
}

class Expect {
  exptects = {
    state: {
      passed: true,
      reason: ""
    }
  };

  /** @type {() => any} */
  test;

  #exptects(expect, callback) {
    if (this.exptects[expect]) {
      console.warn(`Attempted to add another ${expect} to an expect where a ${expect} already exists.`);
      return this;
    }
    this.exptects[expect] = (function (...params) {
      if (this.exptects.state.passed == false) {
        this.globals.log(TESTTYPE.WARNING, [], `Skipping ${expect} due to the test already failing (passed = ${this.exptects.state.passed}, expected 'false')`);
        // Skip over the check if we failed the test anyway.
        return this.exptects;
      }

      let data = callback(this.globals, this.result, ...params);
      this.exptects.state.passed = data.passed;
      this.exptects.state.reason = `(${expect}) ${data.reason}`;
      return this.exptects
    }).bind(this);

    return this;
  }

  from(test_callback) {
    this.globals.log(TESTTYPE.INFO, ["%cRunning test function%c", "color: yellow", ""])
    this.result = noSyncTryCatch(test_callback(this.globals));
    this.globals.log(TESTTYPE.INFO, ["%cTest function completed, running tests on result%c", "color: yellow", ""])
    return this.exptects;
  }

  /**
  *
  * @param {*} globals
  * @param {*} result
  * @param {string} object
  */
  #getObject(globals, result, object) {
    let path = object.split(".");
    let start;
    switch (path.shift()) {
      case "globals": start = globals; break;
      case "result": start = result; break;
      default: throw new Error("#getObject couldn't determine which path to get the object from!");
    }
    let goal = start[path.shift()];
    path.forEach((subPath) => {
      if (goal == undefined) return undefined;
      if (/^\[\d+/.test(subPath)) {
        let index = parseInt(subPath.match(/^\[(\d+)\]/)[1], 10);
        goal = goal[index];
        return;
      }
      goal = goal[subPath];
    })
    return goal;
  }

  #typeOf(obj) {
    return obj?.name ?? typeof obj;
  }

  constructor(globals) {
    this.globals = globals;

    this.#exptects('type', (globals, result, path, type) => {
      this.globals.log(TESTTYPE.INFO, ["%cType Test:%c", "color: lime", ""], `Attempting to see if ${path} is of ${this.#typeOf(type)}`);

      let obj = this.#getObject(globals, result, path);
      let isType = obj instanceof type;

      this.globals.log(isType ? TESTTYPE.SUCCESS : TESTTYPE.ERROR, ["%cType Test:%c", "color: lime", ""], isType ? 'Succeeded' : `Failed: ${path} -> Found '${this.#typeOf(obj)}' expected '${this.#typeOf(type)}'`);
      return {
        passed: isType,
        reason: isType ? '' : `${path} -> Found '${this.#typeOf(obj)}' expected '${this.#typeOf(type)}'`
      }
    });
    this.#exptects('exists', (globals, result, path) => {
      this.globals.log(TESTTYPE.INFO, ["%cExist Test:%c", "color: lime", ""], `Attempting to see if ${path} exists`);

      let obj = this.#getObject(globals, result, path);
      let exists = obj != undefined;

      this.globals.log(exists ? TESTTYPE.SUCCESS : TESTTYPE.ERROR, ["%cType Test:%c", "color: lime", ""], exists ? 'Succeeded' : `Failed: ${path} not found`);
      return {
        passed: exists,
        reason: exists ? '' : `${path} not found'`
      }
    })
  }
}

let test = new Test();
