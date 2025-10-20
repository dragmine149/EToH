/**
 * Custom class for listening to stuff. Could probably use built-in browser events but this is
 * more kinda my style in a way...
 */
class Listeners<Key, Function> {
  #listeners: Map<Key, Function[]>;

  /**
   * Add a listener to call when something happens.
   * @param key The identifier, aka event.
   * @param callback The function to call upon this listener.
   */
  add_listener(key: Key, callback: Function) {
    const array = this.#listeners.get(key) ?? [];
    array.push(callback);
    this.#listeners.set(key, array);
  }

  /**
   * Remove a listener/s from being watched.
   * @param key The identifier, aka event.
   * @param callback The function to call upon this listener. If not exists, will delete all functions
   * under this event/key.
   */
  remove_listener(key: Key, callback?: Function) {
    if (callback) {
      const map = this.#listeners.get(key);
      if (map == undefined) return;
      const index = map.indexOf(callback);
      map.splice(index, 1);
    }

    this.#listeners.delete(key);
  }

  /**
   * Call all the functions waiting for said event.
   * @param key The event triggered.
   * @param details Details to send alongside.
   */
  call_listener(key: Key, ...details: any[]) {
    this.#listeners.get(key)?.forEach((listener) => {
      (listener as any)(...details);
    })
  }

  constructor() {
    this.#listeners = new Map();
  }
}

export { Listeners };
