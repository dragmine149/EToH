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
      case true: consoleMethod = console.log; break;
      case false: consoleMethod = console.error; break;
      case TESTTYPE.INFO: consoleMethod = console.info; break;
      case TESTTYPE.DEBUG: consoleMethod = console.debug; break;
      case TESTTYPE.WARNING: consoleMethod = console.warn; break;
      case TESTTYPE.ERROR: consoleMethod = console.error; break;
      case TESTTYPE.SUCCESS: consoleMethod = console.log; break;
      case TESTTYPE.FAILED: consoleMethod = console.error; break;
      default: consoleMethod = console.log; break;
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
    this.test_count = 0;
    this.test_passed = 0;
    test_function();
  }

  /**
  * Describe a test and what to do.
  * @param {String} test_name The name of the test.
  * @param {() => any)} test_function The function to test.
  */
  test(test_name, test_function) {
    this.test_count += 1;
    this.log(TESTTYPE.INFO, [`%cStarting test:%c`, `color: orange`, ``], `${test_name}`);

    let globals = this.test_data.map((v) => (typeof v == "function") ? v() : v)
    let combinedGlobals = {};
    globals.forEach(global => {
      Object.assign(combinedGlobals, global);
    });;

    let expect = new Expect(combinedGlobals);
    let result = test_function(expect);
    this.test_passed += result.state.passed ? 1 : 0;
    this.log(TESTTYPE.INFO, [`%cFinished test:%c`, `color: orange`, ``], `${test_name}`);
    this.log(result.state.passed ? TESTTYPE.SUCCESS : TESTTYPE.FAILED, [`%cTest results: %c`, `color: orange`, ``], result.state.passed ? `Passed (${this.test_passed}/${this.test_count})!` : `Failed (${this.test_passed}/${this.test_count}): ${result.state.reason}`);
    this.log(TESTTYPE.DEBUG, [`%c-------------------------------------------------%c`, 'color: orange', '']);
  }
}

class Expect {
  exptects = {
    state: {
      passed: true,
      reason: ""
    }
  };

  /** @type {{method: string, arguments: IArguments}[]} */
  log;

  /** @type {() => any} */
  test;

  #exptects(expect, callback, bypassFail) {
    if (this.exptects[expect]) {
      console.warn(`Attempted to add another ${expect} to an expect where a ${expect} already exists.`);
    }
    this.exptects[expect] = (function (...params) {
      if (this.exptects.state.passed == false && !bypassFail) {
        this.globals.log(TESTTYPE.WARNING, [], `Skipping ${expect} due to the test already failing (passed = ${this.exptects.state.passed}, expected 'false')`);
        // Skip over the check if we failed the test anyway.
        return this.exptects;
      }

      let data = callback(this.globals, this.result, ...params);
      this.exptects.state.passed = data.passed;
      this.exptects.state.reason = `(${expect}) ${data.reason}`;
      return this.exptects
    }).bind(this);
  }

  catchLog() {
    this.log = [];
    this._logs = {
      'log': console.log,
      'warn': console.warn,
      'error': console.error,
      'info': console.info,
      'debug': console.debug,
    };

    function consoleBind(method) {
      this.log.push({ method: method, arguments: arguments });
      return this._logs[method].apply(console, arguments);
    }

    console.log = consoleBind.bind(this, 'log');
    console.warn = consoleBind.bind(this, 'warn');
    console.error = consoleBind.bind(this, 'error');
    console.info = consoleBind.bind(this, 'info');
    console.debug = consoleBind.bind(this, 'debug');
  }

  releaseLog() {
    console.log = this._logs['log'];
    console.warn = this._logs['warn'];
    console.error = this._logs['error'];
    console.info = this._logs['info'];
    console.debug = this._logs['debug'];
    this._logs = {};
  }

  from(test_callback) {
    if (!this.#test_type(test_callback, "function")) {
      this.globals.log(TESTTYPE.WARNING, [], "Failed to find function passed in. Skipping test suite");
      this.exptects.state = {
        passed: false,
        reason: 'No test function found'
      }
      return this.exptects.log_no_has("error");
    }

    this.globals.log(TESTTYPE.INFO, ["%cRunning test function%c", "color: yellow", ""])

    this.catchLog();
    this.result = noSyncTryCatch(test_callback.bind(globalThis, this.globals));
    this.releaseLog();

    this.globals.log(TESTTYPE.INFO, ["%cTest function completed, running tests on result%c", "color: yellow", ""])
    if (this.result.error) {
      this.globals.log(TESTTYPE.ERROR, ["%cTest function erroed!%c", "color: yellow", ""], this.result.error);
      this.exptects.state = {
        passed: false,
        reason: `Something failed whilst trying to run the test function: ${this.result.error}`
      };
    }
    this.result = this.result.data;

    return this.exptects.log_no_has("error");
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
    if (typeof obj === "string") return obj;
    return obj?.name ?? typeof obj;
  }

  /**
   * Tests if a variable is of a given type.
   * @param {*} variable - The variable to test.
   * @param {string | Function} type - The type to test against.
   *   Can be a string for primitive types (including "null", "undefined",
   *   "boolean", "number", "string", "symbol", "bigint"), or a constructor function (like Array, Object, custom classes).
   *   Does not currently support complex JSDoc types like union types,
   *   object shapes, etc.
   * @returns {boolean} - True if the variable matches the type, false otherwise.
   */
  #test_type(variable, type) {
    if (typeof type === 'string') {
      // Handle primitive type names
      switch (type) {
        case "null": return variable === null;
        case "undefined": return typeof variable === "undefined";
        case "boolean": return typeof variable === "boolean";
        case "number": return typeof variable === "number";
        case "string": return typeof variable === "string";
        case "symbol": return typeof variable === "symbol";
        case "bigint": return typeof variable === "bigint";
        case "function": return typeof variable === "function";
        case "object": return typeof variable === "object";
        default:
          this.globals.log(TESTTYPE.WARNING, [], `Unknown primitive type string: ${type}`);
          return false;
      }
    }

    if (typeof type === 'function') {
      // Handle constructors/classes
      if (type === Array) return Array.isArray(variable);

      // instanceof works for most other constructors
      try {
        return variable instanceof type;
      } catch (_) {
        // instanceof will throw if variable is a primitive
        return false;
      }
    }

    this.globals.log(TESTTYPE.WARNING, [], `Invalid type argument provided. Must be a string or a constructor. (Got: ${type})`);
    return false;
  }

  #expects_return(passed, test, message) {
    this.globals.log(
      passed ? TESTTYPE.SUCCESS : TESTTYPE.ERROR,
      [`%c${test}:%c`, "color: lime", ""],
      passed ? "Passed" : `Failed: ${message}`
    )
    return {
      passed: passed,
      reason: passed ? '' : message
    }
  }

  constructor(globals) {
    this.globals = globals;

    this.#exptects('type', (globals, result, path, type) => {
      this.globals.log(TESTTYPE.INFO, ["%cType Test:%c", "color: lime", ""], `Attempting to see if ${path} is of ${this.#typeOf(type)}`);

      let obj = this.#getObject(globals, result, path);
      let isType = this.#test_type(obj, type);

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
    });
    this.#exptects('exists_type', (globals, result, path, type) => {
      this.globals.log(TESTTYPE.INFO, ["%cExist Type Test:%c", "color: lime", ""], `Attempting to see if ${path} exists and is of ${this.#typeOf(type)}`);

      let obj = this.#getObject(globals, result, path);
      let exists = obj != undefined;
      this.globals.log(exists ? TESTTYPE.SUCCESS : TESTTYPE.ERROR, ["%cExists Type (Exists) Test:%c", "color: lime", ""], exists ? 'Succeeded' : `Failed: ${path} not found`);
      if (!exists) {
        return {
          passed: false,
          reason: `${path} not found'`
        }
      }

      let isType = this.#test_type(obj, type);
      this.globals.log(isType ? TESTTYPE.SUCCESS : TESTTYPE.ERROR, ["%cExists Type (Type) Test:%c", "color: lime", ""], isType ? 'Succeeded' : `Failed: ${path} -> Found '${this.#typeOf(obj)}' expected '${this.#typeOf(type)}'`);
      return {
        passed: isType,
        reason: isType ? '' : `${path} -> Found '${this.#typeOf(obj)}' expected '${this.#typeOf(type)}'`
      }
    });
    this.#exptects('is', (globals, result, path, goal) => {
      this.globals.log(TESTTYPE.INFO, ["%cIs Test:%c", "color: lime", ""], `Attempting to see if ${path} is ${goal}`);
      let obj = this.#getObject(globals, result, path);
      let is = obj === goal;
      return this.#expects_return(is, "Is", `${path} does not (triple) equal ${goal}`);
    });
    this.#exptects('log_has', (globals, result, type, count) => {
      count = count ? count : 1;
      this.globals.log(TESTTYPE.INFO, ["%cLog Test:%c", "color: lime", ""], `Attempting to see if log has at least ${count} of ${type}`);

      let typeLog = this.log.filter(log => log.method === type);
      let threshold = typeLog.length >= count;

      return this.#expects_return(threshold, "Log", `log does not have at least ${count} of ${type} (found: ${typeLog.length}`);
    });
    this.#exptects('log_no_has', (globals, result, type, count) => {
      count = count ? count : 1;
      this.globals.log(TESTTYPE.INFO, ["%cLog invert Test:%c", "color: lime", ""], `Attempting to see if log has less than ${count} of ${type}`);

      let typeLog = this.log.filter(log => log.method === type);
      let threshold = typeLog.length < count;

      return this.#expects_return(threshold, "Log invert", `log has more of ${type} that expected! (found: ${typeLog.length}, expected: ${count}`);
    });
  }
}

let test = new Test();
