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
    if (this.badgeSearch) {
      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;

      window.addEventListener("keydown", (e) => {
        if ((isMac ? e.metaKey : e.ctrlKey) && e.key === "f") {
          e.preventDefault();
          this.badgeSearch.hidden = false;
          this.badgeSearchInput.focus();
          this.badgeSearchInput.select();
        }
        if (e.key === "Escape") {
          if (!this.badgeSearchInput.matches(':focus')) {
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
      // let table_size =

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
  * Set data for a specific badge when the user is not hovering that badge.
  * @param {String} badge The name of the badge.
  * @param {String} text The text to show in the name field.
  * @param {String} information The text to show in the information field.
  */
  set_data(badge, text, information) {
    let elm = this.badges.get(badge);
    elm.nHover = { text: text, information }

    let badgeName = elm.querySelector("[tag='name']")
    let badgeInformation = elm.querySelector("[tag='info']");

    this.search_data[badgeName.innerHTML] = undefined;
    this.search_data[badgeInformation.innerHTML] = undefined;

    badgeName.innerHTML = text;
    badgeInformation.innerHTML = information;

    this.search_data[badgeName.innerHTML] = badge;
    this.search_data[badgeInformation.innerHTML] = badge;
  }
  /**
  * Set data for a specific badge when the user is hovering that badge.
  * @param {String} badge The name of the badge.
  * @param {String} text The text to show in the name field.
  * @param {String} information The text to show in the information field.
  */
  set_hover(badge, text, information) {
    let elm = this.badges.get(badge);

    this.search_data[elm.hover.text] = undefined;
    this.search_data[elm.hover.information] = undefined;

    elm.hover = { text: text, information };

    this.search_data[elm.hover.text] = text;
    this.search_data[elm.hover.information] = information;
  }
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
  previous_search = [];

  /**
  * Custom search function to do a bit more than the browser.
  * @param {HTMLInputElement} search_elm The element of searching.
  */
  search(search_elm) {
    // this.verbose.log(search_elm);
    // this.verbose.log(search_elm.value);
    let value = search_elm.value; // .toLowerCase();
    // let filteredSearch = Object.entries(this.search_data).map((v) => [v[0].toLowerCase(), v[1]]).filter((v) => v[0].startsWith(value));
    let filteredSearch = Object.entries(this.search_data).filter((v) => v[0].startsWith(value));
    // this.verbose.log(filteredSearch);
    filteredSearch.forEach((badge) => this.#effectElm(this.badges.get(badge[1]), undefined, value));
    // special filter on those that are not in the new search but was in the old to reset them.
    this.previous_search.filter((v) => !filteredSearch.includes(v)).forEach((badge) => this.#effectElm(this.badges.get(badge[1]), undefined, value));

    this.previous_search = filteredSearch;
  }

  /**
  * Effects an element depending on whats happening.
  * @param {HTMLDivElement} elm The element to affect.
  * @param {boolean} hover Is the user hovering us.
  * @param {String} search Search terms.
  */
  #effectElm(elm, hover, search) {
    // Get the children first.
    /** @type {HTMLDivElement} */
    let badgeName = elm.querySelector("[tag='name']")
    /** @type {HTMLDivElement} */
    let badgeInformation = elm.querySelector("[tag='info']");

    // check for hovering.
    elm.isHover = hover == undefined ? elm.isHover : hover;
    elm.search = search == undefined ? elm.search : search;

    // this.verbose.log(elm, hover, elm.isHover, search, elm.search);

    // get the base text.
    /** @type {string} */
    let name_text = elm.isHover ? elm.hover.text : elm.nHover.text;
    if (elm.search != '') {
      name_text = name_text.replace(elm.search, `<span class="search">${elm.search}</span>`);
    }

    let info_text = elm.isHover ? elm.hover.information : elm.nHover.information;

    badgeName.innerHTML = name_text;
    badgeInformation.innerHTML = info_text;
  }

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
      /** @type {HTMLDivElement} */
      let badgeInformation = clone.querySelector("[tag='info']");
      // let badgeCompleted = clone.querySelector("[tag='completed']");
      badgeName.innerHTML = badge;

      // Store data inside `nHover` and `hover`. These are hidden and allow for dynamic stuff.
      clone.nHover = {
        text: badge,
        information: ""
      }
      clone.hover = {
        text: badge,
        information: ""
      }

      // Hovering functions. Dynamic text changing.
      clone.onmouseover = () => this.#effectElm(clone, true);
      clone.onmouseleave = () => this.#effectElm(clone, false);
      // clone.onmouseover = () => {
      //   badgeName.innerHTML = clone.hover.text;
      //   badgeInformation.innerHTML = clone.hover.information;
      // }
      // clone.onmouseleave = () => {
      //   badgeName.innerHTML = clone.nHover.text;
      //   badgeInformation.innerHTML = clone.nHover.information;
      // }
      // clone.highlight = (text) => {
      //   badgeName.innerHTML = clone.hover.text ? clone.hover.text : clone.nHover.text;
      //   if (text == null) return;
      //   badgeName.innerHTML = badgeName.innerHTML.replaceAll(text, `<span class='search'>${text}</span>`);
      // }

      this.search_data[badge] = badge;

      this.badges.set(badge, clone);
    })
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
