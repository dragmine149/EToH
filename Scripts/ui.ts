import { Badge } from "./BadgeManager";

interface UIBadgeData<K extends Badge> {
  /** Function to call to show information in the name field of the ui. */
  name: K['get_name_field'],
  /** Function to call to show information in the info field of the ui. */
  information: K['get_information_field'],
  /** Link to the badge on roblox itself. */
  url: K['link'],
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
  #totalElements: number;
  #completedElements: number;

  #shadow?: ShadowRoot;
  #table?: HTMLTableElement;
  #header?: HTMLSpanElement;
  #badges?: HTMLTableRowElement[];
  #style?: HTMLLinkElement;

  constructor() { super(); }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { }

  connectedCallback() {
    this.#shadow = this.attachShadow({ mode: "open" });
    this.#table = document.createElement("table");
    this.#header = document.createElement("span");
    this.#style = document.createElement("link");
    this.#badges = [];

    this.#shadow.appendChild(this.#style);
    this.#shadow.appendChild(this.#header);
    this.#shadow.appendChild(this.#table);

    this.classList.add("area");
    this.#style.href = "css/shadow_tables.css";
    this.#style.rel = "stylesheet";

    this.#totalElements = 0;
    this.#completedElements = 0;

    this.#updateTable();
  }

  #effectElement(name_data: HTMLTableCellElement, info_span: HTMLSpanElement, hover: boolean, badge: UIBadgeData<K>) {
    // console.log("Hovered row element!");
    name_data.innerText = badge.name(hover);
    info_span.innerText = badge.information(hover);
  }

  #updateCount() {
    if (this.#header == undefined) return;
    let count_data = ``;
    switch (this.count) {
      case Count.None:
        count_data = ``;
        break;
      case Count.Numbers:
        count_data = ` (${this.#completedElements}/${this.#totalElements})`;
        break;
      case Count.Percent:
        count_data = ` (${((this.#completedElements / this.#totalElements) * 100).toFixed(2)}%)`;
        break;
    }

    this.#header.innerText = `${this.#data?.name}${count_data}`;
  }

  #updateTable() {
    // Can't do anything without these two important nodes.
    if (this.#data == undefined) return;
    if (this.#shadow == undefined) return;
    if (this.#table == undefined) return;
    if (this.#header == undefined) return;

    this.#header.title = this.#data.name;
    this.#header.innerText = this.#data.name;

    this.#data.badges.forEach((badge) => {
      const row = document.createElement("tr");
      const name_data = document.createElement("td");
      const info_data = document.createElement("td");
      const info_span = document.createElement("span");
      const info_br = document.createElement("br");
      const info_date = document.createElement("span");

      name_data.innerText = badge.name(false);
      info_span.innerHTML = badge.information();
      info_date.innerText = badge.completed > 0 ? new Date(badge.completed).toLocaleString(undefined, {
        year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
      }) : '';

      row.appendChild(name_data);
      row.appendChild(info_data);
      info_data.appendChild(info_span);
      info_data.appendChild(info_br);
      info_data.appendChild(info_date);

      row.onmouseover = this.#effectElement.bind(this, name_data, info_span, true, badge);
      row.onmouseleave = this.#effectElement.bind(this, name_data, info_span, false, badge);

      this.#table?.appendChild(row);
      this.#badges?.push(row);

      this.#totalElements += 1;
      this.#completedElements += badge.completed > 0 ? 1 : 0;
    });

    this.#updateCount();
  }
}

customElements.define("category-information", CategoryInformation);

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
    }));

  return {
    name: randomName,
    badges: randomBadges,
  };
}

const createCI = () => {
  console.log('creating new element');
  const ci = document.createElement('category-information') as CategoryInformation<Badge>;
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
