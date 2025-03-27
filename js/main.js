/**
* Shows an error message to the user
* @param {string} message The message to show
*/
function showError(message) {
  document.getElementById('error_message').innerText = message;
  document.getElementById('errors').hidden = false;
}

/**
 * @template T
 * @typedef {{
 *   data: T|null;
 *   error: Error|null;
 * }} TryCatchResult
 */

/**
 * Wraps a promise in a try/catch block and returns standardized result
 * @template T
 * @param {Promise<T>} promise - Promise to handle
 * @returns {Promise<TryCatchResult<T>>} Standardized result with data/error
 */
async function tryCatch(promise) {
  try {
    const data = await promise;
    return { data, error: null };
  } catch (error) {
    return { data: null, error: error };
  }
}
