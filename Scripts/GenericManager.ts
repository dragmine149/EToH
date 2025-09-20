type Constructor<T> = new (...args: unknown[]) => T;

class GenericManager<K, T> {
  #items: K[] = [];
  #filters: Record<string, (item: K) => T | T[]> = {};

  #maps: Record<string, Map<T, number[]>> = {};

  /**
  * Get an item from a map. Returns the map keys if no item is defined.
  * @returns A list of keys or the items that have been mapped.
  */
  #mapGetter(map: Map<T, number[]>, item?: T) {
    if (item == undefined) {
      // No item? Return the keys to allow us to view what we can use.
      // Not sorted, we'll let the user deal with sorting.
      return Array.from(map.keys());
    }

    const indexes = map.get(item);
    if (indexes == undefined) return []; // couldn't be found.
    return indexes.map((index) => this.#items[index]); // got to return the items (hence the map). No use otherwise.
  }

  /**
    * Add an item to the map.
    * @param map The map to change.
    * @param key The key of the value to store. Will store the same value under multiple keys if an array is provided.
    * @param value The value to store.
    */
  #mapSetter(map: Map<T, number[]>, key: T | T[], value: number) {
    if (!Array.isArray(key)) {
      key = [key];
    }

    key.forEach(k => {
      // got to have a valid key before it can be inserted.
      if (k == undefined || k == null || Number.isNaN(k)) {
        console.warn(`Invalid key ${k} whilst trying to add to map!`);
        console.debug(`Map:`, map, `value: `, value);
        // console.info(this.#filters);
        return;
      }

      if (!map.has(k)) {
        // if we don't have anything, make a new one.
        map.set(k, [value]);
        return;
      }
      map.get(k)?.push(value);
    });
  }

  #processFilter(filter_name: string, item: K, index: number) {
    // filterFunc will never be undefined because there is a for loop right before this.
    const filterFunc = this.#filters[filter_name];
    // console.info(filterFunc, filter_name, item, index);

    const key = filterFunc(item);
    // console.info(filter_name, key, index, item);
    this.#mapSetter(this.#maps[filter_name], key, index);
  }

  /**
   * Add an item to keep track of.
   * @param item The item to add.
   */
  addItem(item: K) {
    const index = this.#items.push(item);

    // Update the filters to recognise the new item.
    Object.keys(this.#filters)
      .forEach((key) => this.#processFilter(key, item, index - 1));
  }

  addFilter(filter_name: string, callback: (item: K) => T | T[]) {
    if (filter_name.includes(" ")) {
      filter_name = filter_name.replaceAll(" ", "_");
      console.warn(`Filter name (${filter_name}) has '_' instead of ' ' now.`);
    }

    this.#filters[filter_name] = callback;
    this.#maps[filter_name] = new Map();

    const getFunc = this.#mapGetter.bind(this, this.#maps[filter_name]);
    Object.defineProperty(this, filter_name, {
      get: function () { return getFunc },
      set: function () { return; },
    });
  }

  type<C extends Constructor<T>>(ctor: C) {
    return this.#items.filter((item) => item instanceof ctor);
  }
}

export { GenericManager }
