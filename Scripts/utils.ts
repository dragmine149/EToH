
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


/**
 * Tests to see if the user is on a mobile device by looking at certain parameters.
 * @returns An estimate to if the user is on a mobile device or not.
 * @author T3 Chat (GPT-5 mini)
 */
const isMobile = (): boolean => {
  // SSR guard
  if (typeof window === 'undefined' || typeof navigator === 'undefined') {
    return false
  }

  const ua = navigator.userAgent ?? ''
  const uaLower = ua.toLowerCase()

  // basic UA hint for phones/tablets
  const mobileUA = /iphone|ipad|ipod|android|mobile/i

  // modern touch detection and coarse pointer hint
  const hasTouch = (navigator.maxTouchPoints ?? 0) > 0
  const coarsePointer = window.matchMedia?.('(pointer: coarse)').matches ?? false

  const smallScreen = Math.min(window.screen.width, window.screen.height) <= 820

  return mobileUA.test(uaLower) || hasTouch || coarsePointer || smallScreen
}

export { tryCatch, noSyncTryCatch, isMobile };
