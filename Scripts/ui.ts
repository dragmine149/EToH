interface Badge<K> {
  /** Name of the badge */
  name: string,
  /** information of the badge, or custom user defined type. */
  information: string | K,
  /** link to the badge. */
  url: URL,
  /** Link to a wiki page about said badge. */
  wiki?: URL,
  /** Completed date in utc time (via `new Date().getTime()`) */
  completed: number,
}

interface CategoryData<K> {
  /** Name of category */
  name: string,
  /** List of badges that come under this category. */
  badges: Badge<K>[],
}

/**
 * Custom HTMLElement for making a table. Uses shadowDOM for cleaner HTML files.
 * Custom functions allows for easy use. Requires type `K` as a custom user defined element. As an extension
 * to a normal string.
 */
class CategoryInformation<K extends string> extends HTMLElement {
  #data?: CategoryData<K>;
  /** Data stored about the element. */
  set data(data: CategoryData<K> | undefined) {
    this.#data = Object.freeze(data);
    this.#updateTable();
  }
  get data() { return this.#data; }

  /** Whether to display the count of completed vs total in the header or not. */
  set count(v) { this.#count = v; this.#table?.classList[v ? 'add' : 'remove']('count'); }
  get count() { return this.#count; }
  #count: boolean = false;

  #shadow?: ShadowRoot;
  #table?: HTMLTableElement;
  #header?: HTMLSpanElement;
  #badges?: HTMLTableRowElement[];

  constructor() { super(); }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { }

  connectedCallback() {
    this.#shadow = this.attachShadow({ mode: "open" });
    this.#table = document.createElement("table");
    this.#header = document.createElement("span");
    this.#badges = [];

    this.classList.add("area");
    this.#shadow.appendChild(this.#header);
    this.#shadow.appendChild(this.#table);

    this.#updateTable();
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

      name_data.innerText = badge.name;
      info_span.innerHTML = badge.information;
      info_date.innerText = badge.completed > 0 ? new Date(badge.completed).toLocaleString(undefined, {
        year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
      }) : '';

      row.appendChild(name_data);
      row.appendChild(info_data);
      info_data.appendChild(info_span);
      info_data.appendChild(info_br);
      info_data.appendChild(info_date);

      this.#table?.appendChild(row);
      this.#badges?.push(row);
    })
  }
}





// class CategoryInformation extends HTMLElement {
//   #data: TestData;
//   set data(v: TestData) {
//     this.#data = v;

//     if (this.#table == undefined) return;
//     v.towers.forEach((t) => {
//       this.addBadge(t.name, t.information, t.completed);
//     })
//   }
//   get data() { return this.#data; }
//   #shadow?: ShadowRoot;
//   #table?: HTMLTableElement;

//   constructor() {
//     super();
//   }

//   connectedCallback() {
//     console.log(`Made new custom html table!`);

//     this.#shadow = this.attachShadow({ mode: "open" });
//     this.#table = document.createElement("table");
//     const header = document.createElement("th");
//     header.innerText = this.data.name;
//     const row = document.createElement("tr");
//     this.#table.appendChild(row);
//     row.appendChild(header);
//     this.#shadow.appendChild(this.#table);

//     this.data.towers.forEach((t) => {
//       this.addBadge(t.name, t.information, t.completed);
//     });
//   }

//   addBadge(name: string, information: string | URL, completed: number) {
//     const row = document.createElement("tr");
//     const name_data = document.createElement("td");
//     const info_data = document.createElement("td");
//     const info_span = document.createElement("span");
//     const info_br = document.createElement("br");
//     const info_date = document.createElement("span");

//     name_data.innerText = name;
//     info_span.innerHTML = typeof information == 'string' ? information : `<a href=${information.toString()}></a>`;
//     info_date.innerText = completed > 0 ? new Date(completed).toLocaleString(undefined, {
//       year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
//     }) : '';

//     row.appendChild(name_data);
//     row.appendChild(info_data);
//     info_data.appendChild(info_span);
//     info_data.appendChild(info_br);
//     info_data.appendChild(info_date);

//     this.#table?.appendChild(row);
//   }
// }

customElements.define("category-information", CategoryInformation);


// interface TestData {
//   name: string;
//   towers: Badge[];
//   difficulty: number;
//   area: string;
// }

/**
 * A function which generates random testing data.
 */
function random_data(): CategoryData<string> {
  const names = ["Forest Path", "Desert Storm", "Mountain Peak", "Ocean Waves", "City Center"];
  const towerTypes = ["Archer", "Cannon", "Magic", "Ice", "Fire", "Lightning", "Earth"];

  const randomName = names[Math.floor(Math.random() * names.length)];
  const randomBadges = towerTypes
    .sort(() => 0.5 - Math.random())
    .slice(0, Math.floor(Math.random() * 4) + 1)
    .map(towerName => ({
      name: towerName,
      information: `Information about ${towerName}`,
      url: new URL(`https://example.com/${towerName.toLowerCase()}`),
      completed: Math.floor(Math.random() < 0.3 ? -1 : Date.now() - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000)),
    }));

  return {
    name: randomName,
    badges: randomBadges,
  };
}

document.addEventListener('DOMContentLoaded', () => {
  document.getElementById("e")?.addEventListener('click', () => {
    console.log('creating new element');
    const ci = document.createElement('category-information') as CategoryInformation;
    ci.data = random_data();

    document.body.appendChild(ci);
  });
});
