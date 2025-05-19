/*eslint no-unused-vars: "error"*/
/*exported tryCatch, noSyncTryCatch, etohDB, DIFFICULTIES, SUB_LEVELS */

/**
 * @template T
 * @typedef {{
 *   data: T|null;
 *   error: Error|null;
 * }} TryCatchResult
 */

/**
 * Wraps a promise in a try/catch block and returns standardised result
 * @template T
 * @param {Promise<T>} promise - Promise to handle
 * @returns {Promise<TryCatchResult<T>>} Standardised result with data/error
 */
async function tryCatch(promise) {
  try {
    const data = await promise;
    return { data, error: null };
  } catch (error) {
    return { data: null, error: error };
  }
}

/**
 * Same as tryCatch but with no async
 * @template T
 * @param {Promise<T>} promise - Promise to handle
 * @returns {Promise<TryCatchResult<T>>} Standardised result with data/error
 */
function noSyncTryCatch(func) {
  try {
    const data = func();
    return { data, error: null };
  } catch (error) {
    return { data: null, error: error };
  }
}


let etohDB = new Dexie("EToH");
etohDB.version(1).stores({
  towers: `[badge_id+user_id], badge_id, user_id`,
  users: `id, name, display, past, last`
})


const DIFFICULTIES = ["Easy", "Medium", "Hard", "Difficult", "Challenging", "Intense", "Remorseless", "Insane", "Extreme", "Terrifying", "Catastrophic"];
const SUB_LEVELS = [
  { threshold: 0.89, name: "Peak" },
  { threshold: 0.78, name: "High-Peak" },
  { threshold: 0.67, name: "High" },
  { threshold: 0.56, name: "Mid-High" },
  { threshold: 0.45, name: "Mid" },
  { threshold: 0.34, name: "Low-Mid" },
  { threshold: 0.23, name: "Low" },
  { threshold: 0.12, name: "Bottom-Low" },
  { threshold: 0.01, name: "Bottom" },
  { threshold: 0.00, name: "Baseline" }
];
