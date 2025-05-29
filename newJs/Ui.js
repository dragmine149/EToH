/*global badgeManager, dayjs, Verbose */
/*eslint no-undef: "error"*/
/*exported UI */

/**
* @typedef {import('./BadgeManager')}
*/

class UI {
  /** @type {Map<string, HTMLDivElement>} */
  badges;
  /** @type {Map<string, HTMLDivElement>} */
  categories;

  constructor(categories, badge_callback, category_callback) {
    this.badges = new Map();
    this.categories = new Map();
    this.verbose = new Verbose("ETOHUI", '#34A853');

    this.verbose.log("Creating the elements", categories);
    // create the ui elements.
    this.#createBadges();
    this.#createCategories(categories);

    // the root element of all evil.
    // Ignore the above comment, the AI snuck in.
    this.root = document.getElementById("badges");

    this.verbose.log("Adding elements to the correct categories.");
    // then deal with setting the parent elements.
    this.badges.forEach((elm, key) => {
      this.verbose.log("Processing badge (callback): ", key);
      let category = badge_callback(key);
      let catElm = this.categories.get(category);
      this.verbose.log(`Desired category: ${category}`);
      if (!catElm) throw new Error(`Trying to add badge to category '${category}' which was not created`);
      catElm.querySelector("table").appendChild(elm);
    });

    this.categories.forEach((elm, key) => {
      this.verbose.log("Processing category (callback): ", key);
      let parent = category_callback(key);
      if (parent == "root") return this.root.appendChild(elm);
      // if (parent == key) return
      let parentElm = this.categories.get(parent);
      if (!parentElm) throw new Error(`Trying to add category '${key}' to an existing category '${parent}' which doesn't exist`);
      parentElm.appendChild(elm);
    });
  }

  show() { this.root.hidden = false; }
  hide() { this.root.hidden = true; }



  /**
  * Update a badge on the ui.
  * @param {string} name The name of the badge.
  * @param {number?} date The date of badge completion. Leave blank to reset completion.
  */
  update_badge(name, date) {
    let elm = this.badges.get(name);
    let badgeCompleted = elm.querySelector("[tag='completed']");
    badgeCompleted.innerText = new dayjs(date).format('LLL');
    elm.classList[date ? "add" : "remove"]("completed");
  }

  set_data(badge, text, information) {
    let elm = this.badges.get(badge);
    elm.nHover = { text: text, information }

    let badgeName = elm.querySelector("[tag='name']")
    let badgeInformation = elm.querySelector("[tag='info']");

    badgeName.innerText = text;
    badgeInformation.innerText = information;
  }
  set_hover(badge, text, information) {
    let elm = this.badges.get(badge);
    elm.hover = { text: text, information }
  }

  #createBadges() {
    badgeManager.name().forEach((badge) => {
      this.verbose.log("Processing Badge: ", badge);
      if (this.badges.has(badge)) return this.verbose.log("already exists");

      /** @type {HTMLDivElement} */
      // create a clone of the element.
      let clone = document.querySelector("[tag='badge']").cloneNode(true);
      clone.hidden = false;

      // by default, set the badge name to the name,
      let badgeName = clone.querySelector("[tag='name']")
      let badgeInformation = clone.querySelector("[tag='info']");
      // let badgeCompleted = clone.querySelector("[tag='completed']");
      badgeName.innerText = badge;

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
      clone.onmouseover = () => {
        badgeName.innerText = clone.hover.text;
        badgeInformation.innerText = clone.hover.information;
      }
      clone.onmouseleave = () => {
        badgeName.innerText = clone.nHover.text;
        badgeInformation.innerText = clone.nHover.information;
      }

      this.badges.set(badge, clone);
    })
  }

  #createCategories(category_list) {
    category_list.forEach((category) => {
      this.verbose.log("Processing Category: ", category);
      if (this.categories.has(category)) return this.verbose.log("already exists");

      /** @type {HTMLDivElement} */
      let clone = document.getElementById("category").cloneNode(true);
      clone.hidden = false;

      // set the title of the category.
      let title = clone.querySelector("[tag='title']");
      title.innerText = category;

      this.categories.set(category, clone);
    })
  }
}
