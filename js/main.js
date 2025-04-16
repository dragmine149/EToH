
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


let towersDB = new Dexie("Towers");
towersDB.version(1).stores({
  towers: `[badge_id+user_id], badge_id, user_id`,
  users: `[id+name], id, name, played`
})
