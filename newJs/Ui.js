/*global badgeManager, dayjs, Verbose */
/*eslint no-undef: "error"*/
/*exported UI */

/**
* @typedef {import('./BadgeManager')}
*/

class UI {
  /** @type {Map<string, HTMLDivElement>} A map of badges and the elements that they control */
  badges;
  /** @type {Map<string, HTMLDivElement>} A map of categories and the elements they control */
  categories;
  /** @type {string[]} A list of badges that have been loaded. */
  loaded;

  constructor(categories, badge_callback, category_callback) {
    this.badges = new Map();
    this.categories = new Map();
    this.loaded = [];
    this.verbose = new Verbose("ETOHUI", '#34A853');
    this.creator_verbose = new Verbose("ETOHUI_Creator", '#34A853');

    this.creator_verbose.log("Creating the elements", categories);
    // create the ui elements.
    this.#createBadges();
    this.#createCategories(categories);

    // the root element of all evil.
    // Ignore the above comment, the AI snuck in.
    this.root = document.getElementById("badges");

    this.creator_verbose.log("Adding elements to the correct categories.");
    // then deal with setting the parent elements.
    this.badges.forEach((elm, key) => {
      this.creator_verbose.log("Processing badge (callback): ", key);
      let category = badge_callback(key);
      let catElm = this.categories.get(category);
      this.creator_verbose.log(`Desired category: ${category}`);
      if (!catElm) throw new Error(`Trying to add badge to category '${category}' which was not created`);
      catElm.querySelector("table").appendChild(elm);
    });

    this.categories.forEach((elm, key) => {
      this.creator_verbose.log("Processing category (callback): ", key);
      // ignore those categories, which have no badges.
      if (elm.querySelectorAll("[tag='badge']").length <= 0) return;

      let parent = category_callback(key);
      if (parent == "root") return this.root.appendChild(elm);
      // if (parent == key) return
      let parentElm = this.categories.get(parent);
      if (!parentElm) throw new Error(`Trying to add category '${key}' to an existing category '${parent}' which doesn't exist`);
      parentElm.appendChild(elm);
    });

    this.syncSize();

    this.badgeSearch = document.getElementById("badge-search");
    this.badgeSearchInput = document.getElementById("badge-search-input");
    this.badgeSearchCount = document.getElementById("badge-search").querySelector("[tag='search_count']");
    if (this.badgeSearch) {
      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;

      window.addEventListener("keydown", (e) => {
        if ((isMac ? e.metaKey : e.ctrlKey) && e.key === "f") {
          // prevent the search if the search isn't in focus.
          if (!this.badgeSearchInput.matches(':focus')) e.preventDefault();
          this.badgeSearch.hidden = false;
          this.badgeSearchInput.focus();
          this.badgeSearchInput.select();
          this.search(this.badgeSearchInput.value);
        }
        if (e.key === "Escape") {
          if (!this.badgeSearchInput.matches(':focus')) {
            this.searchCleanUp();
            this.badgeSearch.hidden = true;
          }
          this.badgeSearchInput.blur();
        }
      });
    }
  }

  show() { this.root.hidden = false; }
  hide() { this.root.hidden = true; }

  syncSize() {
    this.show();
    this.categories.forEach((elm) => elm.style = ``);
    this.badges.forEach((elm) => elm.dispatchEvent(new Event("mouseover")));
    let height = 0, width = 0;
    function set_size(h, w) { height = Math.max(height, h); width = Math.max(width, w); }

    this.categories.forEach((elm) => {
      set_size(elm.clientHeight, elm.clientWidth);
    });
    // this.categories.forEach((elm) => elm.style = `width: ${width}px; height: ${height}px;`);
    this.categories.forEach((elm) => elm.style = `width: ${width}px;`);

    this.badges.forEach((elm) => elm.dispatchEvent(new Event("mouseleave")));
    this.hide();
  }

  /**
  * Update a badge on the ui.
  * @param {string} name The name of the badge.
  * @param {number?} date The date of badge completion. Leave blank to reset completion.
  */
  update_badge(name, date) {
    let elm = this.badges.get(name);
    let badgeCompleted = elm.querySelector("[tag='completed']");
    badgeCompleted.date = date;
    date = Math.min(badgeCompleted.date, date);
    badgeCompleted.innerHTML = date ? new dayjs(date).format('L LT') : '';
    elm.classList[date ? "add" : "remove"]("completed");

    date ? this.loaded.push(name) : this.loaded.filter((v) => v != name);
  }

  /**
  * Unloads a loaded UI, by hiding itself and setting all the badges to uncompleted.
  */
  unload_loaded() {
    this.loaded.forEach((badge) => this.update_badge(badge, undefined));
    this.hide();
  }

  /** @type {Object.<string, string>} Search data. Key being the search term and the value being the associating badge / category */
  search_data = {};

  /**
  * Sets classes for a specific badge.
  * @param {String} badge The name of the badge.
  * @param {Array[]} name The classes for the text field.
  * @param {Array[]} information The classes for the information field.
  */
  set_classes(badge, name, information) {
    let elm = this.badges.get(badge);
    let badgeName = elm.querySelector("[tag='name']")
    let badgeInformation = elm.querySelector("[tag='information']");

    badgeName.classList.remove(...badgeName.classList);
    name.forEach((n) => badgeName.classList.add(n));

    badgeInformation.classList.remove(...badgeInformation.classList);
    information.forEach((n) => badgeInformation.classList.add(n));
  }

  /** @type [string, string][] */
  previous_search_list = [];
  search_value = "";

  /**
  * Custom search function to do a bit more than the browser.
  * @param {string} value The element of searching.
  */
  search(value) {
    // transform to lower case as it makes everything easier to work with.
    value = value.toLowerCase();
    let filteredSearch = Object.entries(this.search_data).map((v) => [v[0].toLowerCase(), v[1]]).filter((v) => v[0].includes(value));
    // if (filteredSearch.length < 10) this.verbose.log(filteredSearch);

    // just clean up the old search.
    if (this.previous_search_list && this.previous_search_list.length > 0) this.previous_search_list
      // ignore those of this search.
      .filter((v) => !filteredSearch.includes(v))
      // remove the rest.
      .forEach((badge) => this.#effectElm(this.badges.get(badge[1]), undefined, ''));

    // clean up the lists if we don't have anything worth searching for.
    if (value == '') {
      this.previous_search_list = [];
      this.#setSearchIndex(0);
      return;
    }

    // filter the badges depending on the search.
    filteredSearch.forEach((badge) => this.#effectElm(this.badges.get(badge[1]), undefined, value));

    // assign this search to a storage so that we can clean it up when we search again.
    this.previous_search_list = filteredSearch;
    this.search_value = value;
    this.#setSearchIndex();
  }

  searchCleanUp() {
    // just clean up the old search.
    this.previous_search_list
      .forEach((badge) => this.#effectElm(this.badges.get(badge[1]), undefined, ''));
    this.previous_search_list = [];
    // this.#setSearchIndex(0);
  }

  #searchIndex = 0;
  set searchIndex(v) { this.#setSearchIndex(v); }
  /** @param {number} v */
  #setSearchIndex(v) {
    // assume display purposes.
    if (v === undefined) v = this.#searchIndex;
    // get and clear the previous selected badge.
    let badge = this.previous_search_list[this.#searchIndex];
    if (badge) this.#effectElm(this.badges.get(badge[1]), undefined, undefined, false);

    // loop around if we go out of bounds.
    if (v > this.previous_search_list.length - 1) v = 0;
    if (v < 0) v = this.previous_search_list.length;

    // make sure we're in bounds (probably don't need this)
    this.#searchIndex = Math.min(this.previous_search_list.length - 1, Math.max(0, v));
    // and update.
    this.badgeSearchCount.innerHTML = `${this.#searchIndex + 1}/${this.previous_search_list.length}`;

    // show new badge data.
    badge = this.previous_search_list[this.#searchIndex];
    if (badge) {
      let elm = this.badges.get(badge[1]);
      this.#effectElm(elm, undefined, undefined, true);
      elm.scrollIntoView(false);
    }
  }
  get searchIndex() { return this.#searchIndex; }

  next_search() { this.searchIndex += 1; }

  previous_search() { this.searchIndex -= 1; }

  /**
  * Effects an element depending on whats happening.
  * @param {HTMLDivElement} elm The element to affect.
  * @param {boolean} hover Is the user hovering us.
  * @param {String} search Search terms.
  * @param {boolean} selected Selected the search for this item
  */
  #effectElm(elm, hover, search, selected) {
    // Get the children first.
    /** @type {HTMLDivElement} */
    let badgeName = elm.querySelector("[tag='name']")
    /** @type {HTMLDivElement} */
    let badgeInformation = elm.querySelector("[tag='info']");

    // check for hovering.
    elm.isHover = hover == undefined ? elm.isHover : hover;
    elm.search = search == undefined ? elm.search : search;
    elm.selected = selected == undefined ? elm.selected : selected;

    /** @type {Badge} */
    let badgeInfo = badgeManager.name(elm.badge)[0];

    // this.verbose.log(elm, hover, elm.isHover, search, elm.search);

    // get the base text.
    /** @type {string} */
    let name_text = badgeInfo.get_name_field(elm.isHover);
    if (elm.search != '' && elm.search != undefined) {
      let regex = new RegExp(`[${elm.search}]`, "gi");
      name_text = name_text.replaceAll(regex, (match) => {
        return `<span class="search ${elm.selected ? 'selected' : ''}">${match}</span>`;
      });
    }

    let info_text = badgeInfo.get_information_field(elm.isHover);

    badgeName.innerHTML = name_text;
    // this.verbose.log(info_text);
    badgeInformation.innerHTML = info_text;
    // this.verbose.log(badgeInformation.innerHTML);
  }

  onFinishedCreate() { }

  #createBadges() {
    badgeManager.name().forEach((badge) => {
      this.creator_verbose.log("Processing Badge: ", badge);
      if (this.badges.has(badge)) return this.creator_verbose.log("already exists");

      /** @type {HTMLDivElement} */
      // create a clone of the element.
      let clone = document.querySelector("[tag='badge']").cloneNode(true);
      clone.hidden = false;

      // by default, set the badge name to the name,
      /** @type {HTMLDivElement} */
      let badgeName = clone.querySelector("[tag='name']")
      // /** @type {HTMLDivElement} */
      // let badgeInformation = clone.querySelector("[tag='info']");
      // let badgeCompleted = clone.querySelector("[tag='completed']");
      badgeName.innerHTML = badge;
      clone.badge = badge;

      // Hovering functions. Dynamic text changing.
      clone.onmouseover = () => this.#effectElm(clone, true);
      clone.onmouseleave = () => this.#effectElm(clone, false);

      this.search_data[badge] = badge;
      this.badges.set(badge, clone);
    });
    this.onFinishedCreate();
  }

  #createCategories(category_list) {
    category_list.forEach((category) => {
      this.creator_verbose.log("Processing Category: ", category);
      if (this.categories.has(category)) return this.creator_verbose.log("already exists");

      /** @type {HTMLDivElement} */
      let clone = document.getElementById("category").cloneNode(true);
      clone.hidden = false;

      // set the title of the category.
      let title = clone.querySelector("[tag='title']");
      title.innerHTML = category;

      this.categories.set(category, clone);
    })
  }
}
