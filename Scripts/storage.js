var __create = Object.create;
var __getProtoOf = Object.getPrototypeOf;
var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __toESM = (mod, isNodeMode, target) => {
  target = mod != null ? __create(__getProtoOf(mod)) : {};
  const to = isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target;
  for (let key of __getOwnPropNames(mod))
    if (!__hasOwnProp.call(to, key))
      __defProp(to, key, {
        get: () => mod[key],
        enumerable: true
      });
  return to;
};
var __commonJS = (cb, mod) => () => (mod || cb((mod = { exports: {} }).exports, mod), mod.exports);

// node_modules/compress-json/dist/debug.js
var require_debug = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.getType = getType;
  exports.throwUnknownDataType = throwUnknownDataType;
  exports.throwUnsupportedData = throwUnsupportedData;
  function getType(o) {
    return Object.prototype.toString.call(o);
  }
  function throwUnknownDataType(o) {
    throw new TypeError("unsupported data type: " + getType(o));
  }
  function throwUnsupportedData(name) {
    throw new TypeError("unsupported data type: " + name);
  }
});

// node_modules/compress-json/dist/number.js
var require_number = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.s_to_int = s_to_int;
  exports.s_to_big_int = s_to_big_int;
  exports.int_to_s = int_to_s;
  exports.big_int_to_s = big_int_to_s;
  exports.num_to_s = num_to_s;
  exports.int_str_to_s = int_str_to_s;
  exports.s_to_num = s_to_num;
  var i_to_s = "";
  for (let i = 0; i < 10; i++) {
    const c = String.fromCharCode(48 + i);
    i_to_s += c;
  }
  for (let i = 0; i < 26; i++) {
    const c = String.fromCharCode(65 + i);
    i_to_s += c;
  }
  for (let i = 0; i < 26; i++) {
    const c = String.fromCharCode(65 + 32 + i);
    i_to_s += c;
  }
  var N = i_to_s.length;
  var s_to_i = {};
  for (let i = 0; i < N; i++) {
    const s = i_to_s[i];
    s_to_i[s] = i;
  }
  function s_to_int(s) {
    let acc = 0;
    let pow = 1;
    for (let i = s.length - 1; i >= 0; i--) {
      const c = s[i];
      let x = s_to_i[c];
      x *= pow;
      acc += x;
      pow *= N;
    }
    return acc;
  }
  function s_to_big_int(s) {
    let acc = BigInt(0);
    let pow = BigInt(1);
    const n = BigInt(N);
    for (let i = s.length - 1; i >= 0; i--) {
      const c = s[i];
      let x = BigInt(s_to_i[c]);
      x *= pow;
      acc += x;
      pow *= n;
    }
    return acc;
  }
  function int_to_s(int) {
    if (int === 0) {
      return i_to_s[0];
    }
    const acc = [];
    while (int !== 0) {
      const i = int % N;
      const c = i_to_s[i];
      acc.push(c);
      int -= i;
      int /= N;
    }
    return acc.reverse().join("");
  }
  function big_int_to_s(int) {
    const zero = BigInt(0);
    const n = BigInt(N);
    if (int === zero) {
      return i_to_s[0];
    }
    const acc = [];
    while (int !== zero) {
      const i = int % n;
      const c = i_to_s[Number(i)];
      acc.push(c);
      int /= n;
    }
    return acc.reverse().join("");
  }
  function reverse(s) {
    return s.split("").reverse().join("");
  }
  function num_to_s(num) {
    if (num < 0) {
      return "-" + num_to_s(-num);
    }
    let [a, b] = num.toString().split(".");
    if (!b) {
      if (a.includes("e")) {
        const [a1, a2] = a.split("e");
        a = a1;
        b = "0e" + a2;
      } else {
        return int_to_s(num);
      }
    }
    let c;
    if (b) {
      [b, c] = b.split("e");
    }
    a = int_str_to_s(a);
    b = reverse(b);
    b = int_str_to_s(b);
    let str = a + "." + b;
    if (c) {
      str += ".";
      switch (c[0]) {
        case "+":
          c = c.slice(1);
          break;
        case "-":
          str += "-";
          c = c.slice(1);
          break;
      }
      c = int_str_to_s(c);
      str += c;
    }
    return str;
  }
  function int_str_to_s(int_str) {
    const num = +int_str;
    if (num.toString() === int_str && num + 1 !== num && num - 1 !== num) {
      return int_to_s(num);
    }
    return ":" + big_int_to_s(BigInt(int_str));
  }
  function s_to_int_str(s) {
    if (s[0] === ":") {
      return s_to_big_int(s.substring(1)).toString();
    }
    return s_to_int(s).toString();
  }
  function s_to_num(s) {
    if (s[0] === "-") {
      return -s_to_num(s.substr(1));
    }
    let [a, b, c] = s.split(".");
    if (!b) {
      return s_to_int(a);
    }
    a = s_to_int_str(a);
    b = s_to_int_str(b);
    b = reverse(b);
    let str = a + "." + b;
    if (c) {
      str += "e";
      let neg = false;
      if (c[0] === "-") {
        neg = true;
        c = c.slice(1);
      }
      c = s_to_int_str(c);
      str += neg ? -c : +c;
    }
    return +str;
  }
});

// node_modules/compress-json/dist/encode.js
var require_encode = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.encodeNum = encodeNum;
  exports.decodeNum = decodeNum;
  exports.decodeKey = decodeKey;
  exports.encodeBool = encodeBool;
  exports.decodeBool = decodeBool;
  exports.encodeStr = encodeStr;
  exports.decodeStr = decodeStr;
  var number_1 = require_number();
  function encodeNum(num) {
    if (num === Infinity) {
      return "N|+";
    }
    if (num === -Infinity) {
      return "N|-";
    }
    if (Number.isNaN(num)) {
      return "N|0";
    }
    const a = "n|" + (0, number_1.num_to_s)(num);
    return a;
  }
  function decodeNum(s) {
    switch (s) {
      case "N|+":
        return Infinity;
      case "N|-":
        return -Infinity;
      case "N|0":
        return NaN;
    }
    s = s.replace("n|", "");
    return (0, number_1.s_to_num)(s);
  }
  function decodeKey(key) {
    return typeof key === "number" ? key : (0, number_1.s_to_int)(key);
  }
  function encodeBool(b) {
    return b ? "b|T" : "b|F";
  }
  function decodeBool(s) {
    switch (s) {
      case "b|T":
        return true;
      case "b|F":
        return false;
    }
    return !!s;
  }
  function encodeStr(str) {
    const prefix = str[0] + str[1];
    switch (prefix) {
      case "b|":
      case "o|":
      case "n|":
      case "a|":
      case "s|":
        str = "s|" + str;
    }
    return str;
  }
  function decodeStr(s) {
    const prefix = s[0] + s[1];
    return prefix === "s|" ? s.substr(2) : s;
  }
});

// node_modules/compress-json/dist/config.js
var require_config = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.config = undefined;
  exports.config = {
    sort_key: false,
    error_on_nan: false,
    error_on_infinite: false
  };
});

// node_modules/compress-json/dist/memory.js
var require_memory = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.memToValues = memToValues;
  exports.makeInMemoryStore = makeInMemoryStore;
  exports.makeInMemoryCache = makeInMemoryCache;
  exports.makeInMemoryMemory = makeInMemoryMemory;
  exports.addValue = addValue;
  var config_1 = require_config();
  var debug_1 = require_debug();
  var encode_1 = require_encode();
  var number_1 = require_number();
  function memToValues(mem) {
    return mem.store.toArray();
  }
  function makeInMemoryStore() {
    const mem = [];
    return {
      forEach(cb) {
        for (let i = 0; i < mem.length; i++) {
          if (cb(mem[i]) === "break") {
            return;
          }
        }
      },
      add(value) {
        mem.push(value);
      },
      toArray() {
        return mem;
      }
    };
  }
  function makeInMemoryCache() {
    const valueMem = Object.create(null);
    const schemaMem = Object.create(null);
    return {
      getValue(key) {
        return valueMem[key];
      },
      getSchema(key) {
        return schemaMem[key];
      },
      forEachValue(cb) {
        for (const [key, value] of Object.entries(valueMem)) {
          if (cb(key, value) === "break") {
            return;
          }
        }
      },
      forEachSchema(cb) {
        for (const [key, value] of Object.entries(schemaMem)) {
          if (cb(key, value) === "break") {
            return;
          }
        }
      },
      setValue(key, value) {
        valueMem[key] = value;
      },
      setSchema(key, value) {
        schemaMem[key] = value;
      },
      hasValue(key) {
        return key in valueMem;
      },
      hasSchema(key) {
        return key in schemaMem;
      }
    };
  }
  function makeInMemoryMemory() {
    return {
      store: makeInMemoryStore(),
      cache: makeInMemoryCache(),
      keyCount: 0
    };
  }
  function getValueKey(mem, value) {
    if (mem.cache.hasValue(value)) {
      return mem.cache.getValue(value);
    }
    const id = mem.keyCount++;
    const key = (0, number_1.num_to_s)(id);
    mem.store.add(value);
    mem.cache.setValue(value, key);
    return key;
  }
  function getSchema(mem, keys) {
    if (config_1.config.sort_key) {
      keys.sort();
    }
    const schema = keys.join(",");
    if (mem.cache.hasSchema(schema)) {
      return mem.cache.getSchema(schema);
    }
    const key_id = addValue(mem, keys, undefined);
    mem.cache.setSchema(schema, key_id);
    return key_id;
  }
  function addValue(mem, o, parent) {
    if (o === null) {
      return "";
    }
    switch (typeof o) {
      case "undefined":
        if (Array.isArray(parent)) {
          return addValue(mem, null, parent);
        }
        break;
      case "object":
        if (o === null) {
          return getValueKey(mem, null);
        }
        if (Array.isArray(o)) {
          let acc = "a";
          for (let i = 0; i < o.length; i++) {
            const v = o[i];
            const key = v === null ? "_" : addValue(mem, v, o);
            acc += "|" + key;
          }
          if (acc === "a") {
            acc = "a|";
          }
          return getValueKey(mem, acc);
        } else {
          const keys = Object.keys(o);
          if (keys.length === 0) {
            return getValueKey(mem, "o|");
          }
          let acc = "o";
          const key_id = getSchema(mem, keys);
          acc += "|" + key_id;
          for (const key of keys) {
            const value = o[key];
            const v = addValue(mem, value, o);
            acc += "|" + v;
          }
          return getValueKey(mem, acc);
        }
      case "boolean":
        return getValueKey(mem, (0, encode_1.encodeBool)(o));
      case "number":
        if (Number.isNaN(o)) {
          if (config_1.config.error_on_nan) {
            (0, debug_1.throwUnsupportedData)("[number NaN]");
          }
          return "";
        }
        if (Number.POSITIVE_INFINITY === o || Number.NEGATIVE_INFINITY === o) {
          if (config_1.config.error_on_infinite) {
            (0, debug_1.throwUnsupportedData)("[number Infinity]");
          }
          return "";
        }
        return getValueKey(mem, (0, encode_1.encodeNum)(o));
      case "string":
        return getValueKey(mem, (0, encode_1.encodeStr)(o));
    }
    return (0, debug_1.throwUnknownDataType)(o);
  }
});

// node_modules/compress-json/dist/core.js
var require_core = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.compress = compress;
  exports.decode = decode;
  exports.decompress = decompress;
  var debug_1 = require_debug();
  var encode_1 = require_encode();
  var memory_1 = require_memory();
  function compress(o) {
    const mem = (0, memory_1.makeInMemoryMemory)();
    const root = (0, memory_1.addValue)(mem, o, undefined);
    const values = (0, memory_1.memToValues)(mem);
    return [values, root];
  }
  function decodeObject(values, s) {
    if (s === "o|") {
      return {};
    }
    const o = {};
    const vs = s.split("|");
    const key_id = vs[1];
    let keys = decode(values, key_id);
    const n = vs.length;
    if (n - 2 === 1 && !Array.isArray(keys)) {
      keys = [keys];
    }
    for (let i = 2; i < n; i++) {
      const k = keys[i - 2];
      let v = vs[i];
      v = decode(values, v);
      o[k] = v;
    }
    return o;
  }
  function decodeArray(values, s) {
    if (s === "a|") {
      return [];
    }
    const vs = s.split("|");
    const n = vs.length - 1;
    const xs = new Array(n);
    for (let i = 0; i < n; i++) {
      let v = vs[i + 1];
      v = decode(values, v);
      xs[i] = v;
    }
    return xs;
  }
  function decode(values, key) {
    if (key === "" || key === "_") {
      return null;
    }
    const id = (0, encode_1.decodeKey)(key);
    const v = values[id];
    if (v === null) {
      return v;
    }
    switch (typeof v) {
      case "undefined":
        return v;
      case "number":
        return v;
      case "string":
        const prefix = v[0] + v[1];
        switch (prefix) {
          case "b|":
            return (0, encode_1.decodeBool)(v);
          case "o|":
            return decodeObject(values, v);
          case "n|":
          case "N|+":
          case "N|-":
          case "N|0":
            return (0, encode_1.decodeNum)(v);
          case "a|":
            return decodeArray(values, v);
          default:
            return (0, encode_1.decodeStr)(v);
        }
    }
    return (0, debug_1.throwUnknownDataType)(v);
  }
  function decompress(c) {
    const [values, root] = c;
    return decode(values, root);
  }
});

// node_modules/compress-json/dist/helpers.js
var require_helpers = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.trimUndefined = trimUndefined;
  exports.trimUndefinedRecursively = trimUndefinedRecursively;
  function trimUndefined(object) {
    for (const key in object) {
      if (object[key] === undefined) {
        delete object[key];
      }
    }
  }
  function trimUndefinedRecursively(object) {
    trimUndefinedRecursivelyLoop(object, new Set);
  }
  function trimUndefinedRecursivelyLoop(object, tracks) {
    tracks.add(object);
    for (const key in object) {
      if (object[key] === undefined) {
        delete object[key];
      } else {
        const value = object[key];
        if (value && typeof value === "object" && !tracks.has(value)) {
          trimUndefinedRecursivelyLoop(value, tracks);
        }
      }
    }
  }
});

// node_modules/compress-json/dist/index.js
var require_dist = __commonJS((exports) => {
  Object.defineProperty(exports, "__esModule", { value: true });
  exports.config = exports.trimUndefinedRecursively = exports.trimUndefined = exports.addValue = exports.decode = exports.decompress = exports.compress = undefined;
  var core_1 = require_core();
  Object.defineProperty(exports, "compress", {
    enumerable: true, get: function () {
      return core_1.compress;
    }
  });
  Object.defineProperty(exports, "decompress", {
    enumerable: true, get: function () {
      return core_1.decompress;
    }
  });
  var core_2 = require_core();
  Object.defineProperty(exports, "decode", {
    enumerable: true, get: function () {
      return core_2.decode;
    }
  });
  var memory_1 = require_memory();
  Object.defineProperty(exports, "addValue", {
    enumerable: true, get: function () {
      return memory_1.addValue;
    }
  });
  var helpers_1 = require_helpers();
  Object.defineProperty(exports, "trimUndefined", {
    enumerable: true, get: function () {
      return helpers_1.trimUndefined;
    }
  });
  Object.defineProperty(exports, "trimUndefinedRecursively", {
    enumerable: true, get: function () {
      return helpers_1.trimUndefinedRecursively;
    }
  });
  var config_1 = require_config();
  Object.defineProperty(exports, "config", {
    enumerable: true, get: function () {
      return config_1.config;
    }
  });
});

// Scripts/storage.ts
var compressJSON = __toESM(require_dist(), 1);
var maxCacheTime = 7 * 24 * 60 * 60 * 1000;
var MiliSeconds = {
  day: 24 * 60 * 60 * 1000,
  hour: 60 * 60 * 1000,
  minute: 60 * 1000,
  second: 1000,
  get midnight() {
    const now = new Date;
    const midnight = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
    return midnight.getTime() - now.getTime();
  }
};

class DragStorage {
  prefix = "";
  constructor(prefix) {
    this.prefix = prefix;
  }
  #key_loop(callback) {
    for (let index = 0; index < localStorage.length; index++) {
      callback(localStorage.key(index));
    }
  }
  setStorage(name, value, cache = 0) {
    localStorage.setItem(`${this.prefix}-${name}`, value);
    if (cache <= 0)
      return;
    if (cache > maxCacheTime)
      console.warn(`Cache time for ${name} (${cache}ms) exceeds max cache time of ${maxCacheTime}ms. Setting to max cache time.`);
    this.setCache(name, Math.min(cache, maxCacheTime));
  }
  getStorage(name) {
    if (!this.validCache(name)) {
      this.removeStorage(name);
    }
    return localStorage.getItem(`${this.prefix}-${name}`);
  }
  hasStorage(name) {
    if (!this.validCache(name)) {
      return false;
    }
    return localStorage.getItem(`${this.prefix}-${name}`) != null;
  }
  removeStorage(name) {
    localStorage.removeItem(`${this.prefix}-${name}`);
  }
  clearPrefix() {
    this.#key_loop((key) => {
      if (key.startsWith(this.prefix)) {
        localStorage.removeItem(key);
      }
    });
  }
  setCache(name, length) {
    let cache_details = localStorage.getItem(`cache`) ?? JSON.stringify(compressJSON.compress({}));
    let cache_compressed = JSON.parse(cache_details);
    const cache = compressJSON.decompress(cache_compressed);
    cache[`${this.prefix}-${name}`] = new Date().getTime() + length;
    localStorage.setItem(`cache`, JSON.stringify(compressJSON.compress(cache)));
  }
  getCache(name) {
    let cache_details = localStorage.getItem(`cache`) ?? JSON.stringify(compressJSON.compress({}));
    let cache_compressed = JSON.parse(cache_details);
    const cache = compressJSON.decompress(cache_compressed);
    return cache[`${this.prefix}-${name}`];
  }
  validCache(name) {
    const date = this.getCache(name);
    if (!date)
      return true;
    const now = new Date;
    if (now.getTime() < date)
      return true;
    return false;
  }
  listStorage() {
    let items = [];
    this.#key_loop((item) => {
      if (item == "cache")
        return;
      if (item.startsWith(this.prefix)) {
        items.push(item.replace(this.prefix + "-", ""));
      }
    });
    return items;
  }
}
export {
  MiliSeconds,
  DragStorage
};
