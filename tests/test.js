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
  excude = [];
  /** The current test we are working on. */
  current = {
    category: "",
    test: "",
    last: ""
  }

  log(type, prefix, ...params) {
    let consoleMethod;

    switch (type) {
      case TESTTYPE.INFO:
        consoleMethod = console.info;
        break;
      case TESTTYPE.DEBUG:
        consoleMethod = console.log; // console.debug isn't universally styled
        break;
      case TESTTYPE.WARNING:
        consoleMethod = console.warn;
        break;
      case TESTTYPE.ERROR:
        consoleMethod = console.error;
        break;
      case TESTTYPE.SUCCESS: // Added case for SUCCESS
        consoleMethod = console.log; // Often success can just be a regular log
        break;
      case TESTTYPE.FAILED: // Added case for FAILED
        consoleMethod = console.error; // Or you could use console.warn depending on severity
        break;
      default:
        consoleMethod = console.log;
    }

    consoleMethod(...prefix, ...params);

    if (new URL(location).hostname == "localhost") {
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
    this.test_data[this.current.category].before.push(setup);
    this.test_data[this.current.category].before.push({
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
    this.log(TESTTYPE.INFO, [`%cStarting test:%c`, `color: orange`, ``], `${test_name}`);

    let globals = this.test_data[this.current.category].before.map((v) => v())
    let combinedGlobals = {};
    globals.forEach(global => {
      Object.assign(combinedGlobals, global);
    });;

    let expect = new Expect(globals);
    test_function(expect);
  }
}

class Expect {
  /** @type {Object.<string, {
    info: any, hit: bool, condition: (user, globals, result) => [bool, string], reason: string
  }>} */
  exptects = {};

  /** @type {() => any} */
  test;

  #exptects(expect, condition, info) {
    if (this.exptects[expect]) {
      console.warn(`Attempted to add another ${expect} to an expect where a ${expect} already exists.`);
      return this;
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

  #verify_test(globals, info) {
    Object.values(this.exptects).forEach((v) => {
      let r = v.condition(globals, info);
      v.hit = r[0];
      v.reason = r[1];
    })
  }

  from(test_callback) {
    let results = noSyncTryCatch(test_callback(this.globals));


    return this.exptects;
  }

  // execute(globals) {
  //   try {
  //     let result = this.test(globals);
  //     this.#verify_test(globals, result);
  //   }
  //   catch (e) {
  //     this.#verify_test(globals, e);
  //   }

  //   let success = true;
  //   let msg = "";
  //   Object.entries(this.exptects).forEach(([key, info], index) => {
  //     console.log(index);
  //     success = info.hit ? success : false;
  //     if (!info.hit) msg += `${index} -> Condition ${key} was not met: ${info.reason}\n`;
  //   });

  //   return [success, msg];
  // }

  constructor(globals) {
    this.globals = globals;

    this.#newExpect("custom", (user, globals, result) => user(globals, result));

    this.#newExpect('type', (v, globals, t) => {
      if (t == undefined) return [false, `Found result of 'from' to be undefined.`];
      return v(t) ? [true, ''] : [false, `Expected type ${typeof (v)} found type ${typeof (t)}`]
    });
    this.#newExpect('error',
      /** @param {Error} v @param {Error} i  */
      (v, globals, i) =>
        (v.name === i.name && v.message === i.message && v.stack !== i.stack)
          ? [true, '']
          : [false, `Expected error with name "${i.name}" and message "${i.message}", but got name "${v.name}" and message "${v.message}".`]
    );
    this.#newExpect('global_exists', (user, globals, _) => user(globals))
  }
}

let test = new Test();
