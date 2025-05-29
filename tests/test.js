/*global noSyncTryCatch*/
/*eslint no-unused-vars: "error"*/
/*exported Test, test, other_file_test */

let waiting_timeout;

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
  }

  /**
  * Runs this code before every test.
  * @param {() => {}} setup The code to run. Must return a json of globals to use in the test.
  */
  before(setup) {
    this.test_data.push(setup);
  }

  /**
  * Describe a new category of tests.
  * @param {String} category_name The name of the category.
  * @param {() => any} test_function The function for all the tests.
  * @param {boolean} ignore Tell the server to ignore this test whilst processing tests.
  * @returns {{exclude: () => any, include: () => any}} Functions that can further affect the test suite.
  */
  describe(category_name, test_function, ignore) {
    this.log(TESTTYPE.INFO, [`%cStarting test suite:%c`, `color: cyan`, ``], `${category_name}`, ignore ? `(.gitignore)` : ``);

    clearTimeout(waiting_timeout);
    this.test_data = [];
    this.test_count = 0;
    this.test_passed = 0;
    test_function();
    this.log(TESTTYPE.INFO, [`%cFinished test suite:%c`, `color: cyan`, ``], category_name, this.test_count == this.test_passed ? `Passed (${this.test_passed}/${this.test_count})!` : `Failed (${this.test_passed}/${this.test_count})`);
    waiting_timeout = setTimeout(() => window.areTestsFinished = true, 10000); // 10 seconds to finish all tests.
  }

  /**
  * Describe a test and what to do.
  * @param {String} test_name The name of the test.
  * @param {() => any)} test_function The function to test.
  */
  test(test_name, test_function) {
    this.test_count += 1;
    this.log(TESTTYPE.INFO, [`%cStarting test:%c`, `color: orange`, ``], `${test_name}`);

    let globals = this.test_data.map((v) => (typeof v == "function") ? v() : v);
    let combinedGlobals = {};
    globals.forEach(global => {
      Object.assign(combinedGlobals, global);
    });;
    combinedGlobals.log = this.log;

    let expect = new Expect(combinedGlobals);
    let result = test_function(expect);
    if (result.state.passed) {
      // only do this on pass, because we have other issues to deal with first.
      let errors = expect.log.filter((v) => v.method == 'error');
      let throws = expect.log.filter((v) => v.method == 'throw');
      if (errors.length > 0 || throws.length > 0) {
        result.state = {
          passed: false,
          reason: `Found '${errors.length} errors, ${throws.length} throws' that were not expected during testing.`
        }
      }
    }

    this.test_passed += result.state.passed ? 1 : 0;
    this.log(TESTTYPE.INFO, [`%cFinished test:%c`, `color: orange`, ``], `${test_name}`);
    this.log(result.state.passed ? TESTTYPE.SUCCESS : TESTTYPE.FAILED, [`%cTest results:%c`, `color: orange`, ``], result.state.passed ? `Passed (${this.test_passed}/${this.test_count})!` : `Failed (${this.test_passed}/${this.test_count}): ${result.state.reason}`);
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
  paths = {};

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

    function consoleBind() {
      let args = [...arguments];
      let method = args.shift();
      this.log.push({ method: method, arguments: args });
      return this._logs[method].apply(console, args);
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
      return this.exptects;
    }

    this.globals.log(TESTTYPE.INFO, ["%cRunning test function%c", "color: yellow", ""])

    this.catchLog();
    this.result = noSyncTryCatch(test_callback.bind(globalThis, this.globals));
    this.releaseLog();

    this.globals.log(TESTTYPE.INFO, ["%cTest function completed, running tests on result%c", "color: yellow", ""])
    if (this.result.error) {
      this.globals.log(TESTTYPE.WARNING, ["%cTest function threw!%c", "color: yellow", ""], this.result.error);
    }
    //   this.globals.log(TESTTYPE.ERROR, ["%cTest function errored!%c", "color: yellow", ""], this.result.error);
    //   this.exptects.state = {
    //     passed: false,
    //     reason: `Something failed whilst trying to run the test function: ${this.result.error}`
    //   };
    if (this.result.error) this.log.push({ method: 'throw', arguments: this.result.error });
    this.result = this.result.data;

    return this.exptects;
  }

  /**
  *
  * @param {*} globals
  * @param {*} result
  * @param {string} object
  */
  #getObject(globals, result, object) {
    if (this.paths[object]) return this.paths[object];
    let path = object.split(".");
    let start;
    switch (path.shift()) {
      case "globals": start = globals; break;
      case "result": start = result; break;
      default: throw new Error("#getObject couldn't determine which path to get the object from!");
    }
    if (path.length == 0) return start;

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
    this.paths[object] = goal;
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
      } catch {
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
      return this.#expects_return(is, "Is", `${path} does not (triple) equal ${goal} (Found: ${obj})`);
    });
    this.#exptects('len', (globals, result, path, len) => {
      this.globals.log(TESTTYPE.INFO, ["%cLength Test:%c", "color: lime", ""], `Attempting to see if ${path} is ${len} long`);
      let obj = this.#getObject(globals, result, path);
      let length = obj.length === len;
      return this.#expects_return(length, "Length", `${path} length is not equal to ${len} (Found: ${obj.length})`);
    });

    this.#exptects('expect', (globals, result, type, message, count) => {
      count ||= 1;
      this.globals.log(TESTTYPE.INFO, ["%cExpect Test", "color: lime", ""], `Checking logs for ${count} ${type} with msg: ${message}`);

      let typeLog = this.log.filter(log => log.method === type && Array.from(log.arguments).join(" ") === message);
      let equalCount = typeLog.length == count;
      this.log = this.log.filter((v) => !typeLog.includes(v)); // remove from log.

      return this.#expects_return(equalCount, `Expect Test`, `Expected ${count} ${type} with msg: ${message}. Found ${typeLog.length} instead`);
    });

    this.#exptects('catch_throw', (globals, result, expected, message, count) => {
      count ||= 1;
      this.globals.log(TESTTYPE.INFO, ["%cCatch Throw Test%c", "color: lime", ""], `Checking logs for throw of type ${this.#typeOf(expected)} with msg: ${message}`);

      let typeLog = this.log.filter(log => log.method === "throw" && this.#test_type(log.arguments, expected) && log.arguments.message == message);
      let equalCount = typeLog.length == count;
      this.log = this.log.filter((v) => !typeLog.includes(v)); // remove from log.

      return this.#expects_return(equalCount, `Catch Throw Test`, `Expected ${count} throw with type '${this.#typeOf(expected)}' with msg: ${message}. Found ${typeLog.length} instead`);
    })
  }
}

let test = new Test();

function other_file_test() {
  return document.createElement("a");
}
