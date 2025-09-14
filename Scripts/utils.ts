
// Types for the result object with discriminated union
interface Success<T> {
  data: T;
  error: null;
}

interface Failure<E> {
  data: null;
  error: E;
}

type Result<T, E = Error> = Success<T> | Failure<E>;

/**
 * Wraps a promise in a try/catch block and returns standardised result
 * @template T
 * @param { Promise<T>} promise - Promise to handle
 * @returns {Promise<TryCatchResult<T>>} Standardised result with data/error
 */
async function tryCatch<T, E = Error>(
  promise: Promise<T>,
): Promise<Result<T, E>> {
  try {
    const data = await promise;
    return { data, error: null };
  } catch (error) {
    return { data: null, error: error as E };
  }
}

/**
 * Same as tryCatch but with no async
 * @param func function to handle
 * @returns Standardised result with data/error
 */
function noSyncTryCatch<T, E = Error>(func: () => T): Result<T, E> {
  try {
    const data = func();
    return { data, error: null };
  } catch (error) {
    return { data: null, error: error as E };
  }
}

export { tryCatch, noSyncTryCatch };
