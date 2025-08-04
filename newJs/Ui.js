/*global badgeManager, Verbose */
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

  /**
  * Create a new UI whilst doing a lot of js setup for it.
  * @param {string[]} categories A list of categories to make.
  * @param {(badge_name: string) => string} badge_callback What category a specific badge should be added to.
  * @param {(category_name: string) => string} category_callback What category should be the parent of this category.
  */
  constructor(categories, badge_callback, category_callback) {
    this.badges = new Map();
    this.categories = new Map();
    this.loaded = [];
    this.verbose = new Verbose("ETOHUI", '#34A853');
    this.creator_verbose = new Verbose("ETOHUI_Creator", '#34A853');

    this.creator_verbose.log("Creating the elements", categories);
    // create the ui elements.
    this.#createBadges();
    // this.#createCategories(categories);

    // the root element of all evil.
    // Ignore the above comment, the AI snuck in.
    this.root = document.getElementById("badges");

    this.creator_verbose.log("Adding elements to the correct categories.");
    let defaultCategory = this.setCategory("default");
    Array.from(this.badges.keys()).forEach((key) => {
      let category = badge_callback(key);
      let parent = category_callback(category);
      if (parent == "root") parent = "default";
      defaultCategory.addBadges(key, category, parent);
    });

    // Stuff to do with the searching system.
    this.badgeSearch = document.getElementById("badge-search");
    this.badgeSearchInput = document.getElementById("badge-search-input");
    this.badgeSearchCount = document.getElementById("badge-search").querySelector("[tag='search_count']");
    if (this.badgeSearch) {
      window.addEventListener("keydown", (e) => {
        // NOTE: Users can either do (ctrl/cmd/win/meta) + f. Partially due to no widely supported (not deprecated) js web standard function for this.
        if ((e.metaKey || e.ctrlKey) && e.key === "f") {
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
    // separator comment as its hard to see.
  }

  show() { this.root.hidden = false; this.load_category("default"); }
  hide() { this.root.hidden = true; }

  syncSize() {
    // this.show();
    this.categories.forEach((elm) => elm.style = ``);
    this.badges.forEach((elm) => elm.dispatchEvent(new Event("mouseover")));
    let height = 0, width = 0;
    function set_size(h, w) { height = Math.max(height, h); width = Math.max(width, w); console.log(width); }

    this.categories.forEach((elm) => {
      set_size(elm.clientHeight, elm.clientWidth);
    });
    // this.categories.forEach((elm) => elm.style = `width: ${width}px; height: ${height}px;`);
    this.categories.forEach((elm) => elm.style = `width: ${width}px;`);

    this.badges.forEach((elm) => elm.dispatchEvent(new Event("mouseleave")));
    // this.hide();
  }

  reset_new() {
    document.querySelectorAll(".completed.new").forEach((node) => node.classList.remove("new"));
  }

  /**
  * Update a badge on the ui.
  * @param {string} name The name of the badge.
  * @param {number?} date The date of badge completion. Leave blank to reset completion.
  * @param {bool} new_since How the badge been claimed since we last loaded the data.
  */
  update_badge(name, date, new_since) {
    let elm = this.badges.get(name);
    let badgeCompleted = elm.querySelector("[tag='completed']");
    badgeCompleted.date = date;
    date = Math.min(badgeCompleted.date, date);
    badgeCompleted.innerHTML = date ? new Date(date).toLocaleString(undefined, {
      year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
    }) : '';
    elm.classList[date ? "add" : "remove"]("completed");
    if (date && new_since) elm.classList.add("new");

    if (!elm.counted) {
      // this.verbose.log(elm.category);
      let cat = this.categories.get(elm.category);
      if (cat) {
        let title = cat.querySelector("[tag='title']");
        let count = Number(title.style.getPropertyValue("--count"));
        count += date ? 1 : 0;
        title.style.setProperty("--count", count);

        // elm.parentNode.style.setProperty("--count", elm.parentNode.style.getPropertyValue("--count") + date ? 1 : 0);
        // this.verbose.log(title, title.style.getPropertyValue("--count"));
        elm.counted = true;
      }
    }

    date ? this.loaded.push(name) : this.loaded.filter((v) => v != name);
  }

  /**
  * Unloads a loaded UI, by hiding itself and setting all the badges to uncompleted.
  */
  unload_loaded() {
    this.badges.forEach((_, k) => { this.update_badge(k, undefined, undefined); _.counted = false });
    this.categories.forEach((node) => {
      node.querySelector("[tag='title']").style.setProperty("--count", 0);
    });
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

  hover = null;

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

    if (hover === true) {
      if (this.hover == elm) return; // just don't update it if we are already hovering it.
      this.hover = elm;
    }
    if (hover === false) {
      this.hover = null;
    }

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

  #createBadges() {
    badgeManager.name().forEach((badge) => {
      this.creator_verbose.log("Processing Badge: ", badge);
      if (this.badges.has(badge)) return this.creator_verbose.log("already exists");

      /** @type {HTMLDivElement} */
      // create a clone of the element.
      let clone = document.querySelector("[tag='badge']").cloneNode(true);
      clone.hidden = false;
      clone.style = "";

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
  }

  /**
  * Create the node (table) for the desired category. Result is saved in this.categories under the provided name.
  * @param {string} category The list (or name) of chategories to display.
  * @param {HTMLDivElement} parent_node The parent node that this category goes under.
  */
  #createCategory(category, parent_node) {
    this.creator_verbose.log("Processing Category: ", category);
    if (this.categories.has(category)) {
      this.creator_verbose.log("already exists");
      return this.categories.get(category);
    }
    if (Object.keys(this.display_categories).includes(category)) {
      return;
    }

    /** @type {HTMLDivElement} */
    let clone = document.getElementById("category").cloneNode(true);
    clone.hidden = false;

    // set the title of the category.
    let title = clone.querySelector("[tag='title']");
    title.innerHTML = category;
    title.title = category;

    this.categories.set(category, clone);
    if (parent_node == undefined) parent_node = this.root;
    parent_node.appendChild(clone);
    return clone;
  }

  /** @typedef {{ data: { [category: string]: string[] }, parents: { [category: string]: string } }} ParentCategories */
  /** @type {{[filter: string]: ParentCategories}} */
  display_categories = {};

  /**
  * Add badges to the category.
  * @param {string} cat_name The name of the category to add stuff to.
  * @param {{[category_name: string]: string}} parents The parent reference list.
  * @param {string|string[]} badges Name of badges.
  * @param {string} name The sub*-category name.
  * @param {string} parent The parent they belong under.
  */
  #addBadges(cat_name, parents, badges, name, parent) {
    this.creator_verbose.log(`Adding: `, badges, `to ${name}, ${parent}`);
    // defaults set up so user provides less.
    if (!Array.isArray(badges)) badges = [badges]; // single badge support
    if (name === undefined || name == null || name == '') name = cat_name; // default to root
    if (parent === undefined || parent == null || parent == '') parent = cat_name;
    // if (!parents[name]) parents[name] = parent; // make sure our parent exists.

    let pNode = this.#createCategory(parent, this.categories.get(parents[parent]));
    this.#createCategory(name, pNode);

    let path = parents[name];
    if (!path) {
      path = `${parents[parent]}.${name}`;
      parents[name] = path;
      if (!parents[parent]) {
        path = `${cat_name}.${name}`;
        parents[parent] = `${cat_name}`;
        parents[name] = path;
      }
    }

    // add all badges into the route node under their specific name.
    // and this syntax... wtf js.
    (this.display_categories[cat_name].data[name] ||= []).push(...badges);

    // return itself for easy to continue usage.
    return {
      addBadges:
        /**
        * @param {string|string[]} badges Name of badges.
        * @param {string} name The sub*-category name.
        * @param {string} parent The parent they belong under.
        */
        (badges, name, parent) => this.#addBadges(cat_name, parents, badges, name, parent)
    }
  }

  /**
  * Set a category containing information about how to modify the display.
  * @param {string} cat_name of the category.
  */
  setCategory(cat_name) {
    this.display_categories[cat_name] = { parents: {}, data: {} };
    // let parents = {};
    return {
      addBadges:
        /**
        * @param {string|string[]} badges Name of badges.
        * @param {string} name The sub*-category name.
        * @param {string} parent The parent they belong under.
        */
        (badges, name, parent) => this.#addBadges(cat_name, this.display_categories[cat_name].parents, badges, name, parent)
    }
  }

  //eslint-disable-next-line no-unused-vars
  onCategoryLoad(_) { }


  current_category = "default";
  /**
  * Display a preset category.
  * @param {string} category_name
  */
  load_category(category_name) {
    let data = this.display_categories[category_name];
    let categories = data.parents;
    this.verbose.log(categories);
    this.current_category = category_name;

    // sort out node visibility first.
    let categoryCategories = Object.keys(categories).flatMap((v) => v)
    this.categories.forEach((value, key) => {
      value.hidden = !categoryCategories.includes(key);
    });
    this.search_data = {};

    // now time for the badges.
    Object.entries(data.data).forEach(([key, value]) => {
      let node = this.categories.get(key);
      let completed = 0;
      let hidden = 0;
      value.forEach((badge) => {
        let child = this.badges.get(badge);
        hidden += child.classList.contains("locked") || child.classList.contains("mini-hidden") ? 1 : 0;
        completed += child.classList.contains("completed") ? 1 : 0;
        // this.verbose.log(hidden);
        node.querySelector("[tag='badges']").appendChild(child);
        badgeManager.name(badge)[0].search(this.search_data);
        child.category = key;
      });

      let title = node.querySelector("[tag='title']");
      // title.innerHTML = `${title.title} (${completed}/${value.length})`;
      title.style.setProperty("--count", completed);
      title.style.setProperty("--hidden", hidden);
      title.style.setProperty("--total", value.length);
      this.onCategoryLoad(key);
    });

    this.syncSize();
  }
}
