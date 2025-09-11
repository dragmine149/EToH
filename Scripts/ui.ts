import { Badge } from "./BadgeManager";

interface UIBadgeData<K extends Badge> {
  /** Function to call to show information in the name field of the ui. */
  name: K['get_name_field'],
  /** Function to call to show information in the info field of the ui. */
  information: K['get_information_field'],
  /** Link to the badge on roblox itself. */
  url: K['link'],
  /** Badge ID */
  id: K['id'],
  /** Link to a wiki page about said badge. */
  wiki?: URL,
  /** Completed date in utc time (via `new Date().getTime()`) */
  completed: number,
}

interface CategoryData<K extends Badge> {
  /** Name of category */
  name: string,
  /** List of badges that come under this category. */
  badges: UIBadgeData<K>[],
}

enum Count {
  None, Numbers, Percent
}

/**
 * Custom HTMLElement for making a table. Uses shadowDOM for cleaner HTML files.
 * Custom functions allows for easy use. Requires type `K` as a custom user defined element. As an extension
 * to a normal string.
 */
class CategoryInformation<K extends Badge> extends HTMLElement {
  #data?: CategoryData<K>;
  /** Data stored about the element. */
  set data(data: CategoryData<K> | undefined) {
    this.#data = Object.freeze(data);
    this.#updateTable();
  }
  get data() { return this.#data; }

  /** Whether to display the count of completed vs total in the header or not. */
  set count(v) { this.#count = v; }
  get count() { return this.#count; }
  #count: Count = Count.Numbers;
  /** Total number of elements we're looking after */
  #totalElements: number;
  /** Number of elements where badge.completed > 0 */
  #completedElements: number;

  /// Contains quick references to different children for global use.
  #shadow?: ShadowRoot;
  #table?: HTMLTableElement;
  #header?: HTMLSpanElement;
  badges?: Map<number, BadgeInformation<K>>;
  #style?: HTMLLinkElement;

  constructor() { super(); }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { }

  connectedCallback() {
    // make the base required data.
    this.#shadow = this.attachShadow({ mode: "open" });
    this.#table = document.createElement("table");
    this.#header = document.createElement("span");
    this.#style = document.createElement("link");
    this.badges = new Map();

    // sort out shadow children
    this.#shadow.appendChild(this.#style);
    this.#shadow.appendChild(this.#header);
    this.#shadow.appendChild(this.#table);

    // sort out styles
    this.classList.add("area");
    this.#style.href = "css/shadow_tables.css";
    this.#style.rel = "stylesheet";

    // reset counters
    this.#totalElements = 0;
    this.#completedElements = 0;

    // and update stuff.
    this.#updateTable();
  }

  /**
   * Formats a string to display counted data.
   * @param completed The completed element count.
   * @param total The total element count.
   * @returns A formatted string based off Count enum.
   */
  #countString(completed: number, total: number) {
    if (this.count == Count.Numbers) return ` (${completed}/${total})`;
    if (this.count == Count.Percent) {
      const percentage = (total === 0) ? 0 : ((completed / total) * 100);
      return ` (${percentage.toFixed(2)}%)`;
    }
    // Also accounts for Count.None
    return ``;
  }

  /**
   * Updates the count display.
   */
  #updateCount() {
    if (!this.#header || !this.#data) return;

    const count_data = this.#countString(this.#completedElements, this.#totalElements);
    this.#header.innerText = `${this.#data.name}${count_data}`;
  }

  /**
   * Update the table with all the badges.
   */
  #updateTable() {
    // Can't do anything without these two important nodes.
    if (!this.#data || !this.#shadow || !this.#table || !this.#header || !this.badges) return;

    // set header as this is easy and can get out of the way.
    this.#header.title = this.#data.name;
    this.#header.innerText = this.#data.name;

    // for every badge.
    this.#data.badges.forEach((badge) => {
      if (this.badges!.has(badge.id)) return;

      const row = document.createElement("badge-info") as BadgeInformation<K>;
      row.data = badge;

      // add to main table and storage.
      this.#table!.appendChild(row);
      this.badges!.set(badge.id, row);

      // increment counters.
      this.#totalElements += 1;
      this.#completedElements += badge.completed > 0 ? 1 : 0;
    });

    // update the ui.
    this.#updateCount();
  }
}

class BadgeInformation<K extends Badge> extends HTMLElement {
  #data?: UIBadgeData<K>;
  /** Data stored about the element. */
  set data(data: UIBadgeData<K> | undefined) {
    this.#data = Object.freeze(data);
    this.#updateRow();
  }
  get data() { return this.#data; }

  /// Contains quick references to different children for global use.
  #shadow?: ShadowRoot;
  #row?: HTMLTableRowElement;
  #name_field?: HTMLTableCellElement;
  #info_field?: HTMLTableCellElement;
  #info_data?: HTMLSpanElement;
  #info_br?: HTMLBRElement;
  #info_comp?: HTMLSpanElement;
  #style?: HTMLLinkElement;

  constructor() { super(); }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { }

  connectedCallback() {
    this.#shadow = this.attachShadow({ mode: "open" });
    this.#row = document.createElement("tr");
    this.#name_field = document.createElement("td");
    this.#info_field = document.createElement("td");
    this.#info_data = document.createElement("span");
    this.#info_br = document.createElement("br");
    this.#info_comp = document.createElement("span");
    this.#style = document.createElement("link");

    // sort out shadow children
    this.#shadow.appendChild(this.#style);
    this.#shadow.appendChild(this.#row);

    // sort out normal children.
    this.#row.appendChild(this.#name_field);
    this.#row.appendChild(this.#info_field);
    this.#info_field.appendChild(this.#info_data);
    this.#info_field.appendChild(this.#info_br);
    this.#info_field.appendChild(this.#info_comp);

    // sort out styles.
    // this.#style.href = "css/shadow_tables.css";
    this.#style.rel = "stylesheet";

    this.#updateRow();
  }

  /**
   * Update element (and over stuff eventually) when we hover.
   * @param hover Is user hover?
   */
  #effectElement(hover: boolean) {
    this.#name_field!.innerText = this.#data!.name(hover);
    this.#info_data!.innerText = this.#data!.information(hover);
  }

  /**
   * Public function to update the completed date of this badge.
   * @param completed Optional completed unix timestamp (`new Date().getTime()`)
   */
  updatedCompleted(completed?: number) {
    if (!this.#info_comp) return;

    this.#info_comp.innerText = completed && completed > 0 ? new Date(completed).toLocaleString(undefined, {
      year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
    }) : '';
  }

  /**
   * Update row information of the badge. (and a lot of related stuff)
   */
  #updateRow() {
    if (!this.#row || !this.#name_field || !this.#info_field || !this.#info_data || !this.#info_br || !this.#info_comp || !this.#data) return;

    // set the fields default values so something exists.
    this.#name_field.innerText = this.#data.name(false);
    this.#info_data.innerHTML = this.#data.information();
    this.#info_comp.innerText = this.#data.completed > 0 ? new Date(this.#data.completed).toLocaleString(undefined, {
      year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
    }) : '';

    // sort out external events.
    this.#row.onmouseover = this.#effectElement.bind(this, true);
    this.#row.onmouseleave = this.#effectElement.bind(this, false);
  }
}


customElements.define("category-info", CategoryInformation);
customElements.define("badge-info", BadgeInformation);

/**
 * A function which generates random testing data.
 */
function random_data(): CategoryData<Badge> {
  const names = ["Forest Path", "Desert Storm", "Mountain Peak", "Ocean Waves", "City Center"];
  const towerTypes = ["Archer", "Cannon", "Magic", "Ice", "Fire", "Lightning", "Earth"];

  const randomName = names[Math.floor(Math.random() * names.length)];
  const randomBadges = towerTypes
    .sort(() => 0.5 - Math.random())
    .slice(0, Math.floor(Math.random() * 4) + 1)
    .map(towerName => ({
      name: (hover: boolean) => towerName + (hover ? " (Hovered)" : ""),
      information: (hover: boolean) => `Information about ${towerName} ` + (hover ? " (Hovered)" : ""),
      url: `https://example.com/${towerName.toLowerCase()}`,
      completed: Math.floor(Math.random() < 0.3 ? -1 : Date.now() - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000)),
      id: Math.floor(Math.random() * 1000),
    }));

  return {
    name: randomName,
    badges: randomBadges,
  };
}

const createCI = () => {
  console.log('creating new element');
  const ci = document.createElement('category-info') as CategoryInformation<Badge>;
  ci.data = random_data();
  ci.count = Math.random() >
    0.33 ? Count.None : (Math.random() > 0.66 ? Count.Numbers : Count.Percent);
  document.body.appendChild(ci);
}

document.addEventListener('DOMContentLoaded', () => {
  document.getElementById("e")?.addEventListener('click', createCI);
});

for (let i = 0; i < 2; i++) {
  createCI();
}
