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

interface CategoryData {
  /** Name of category */
  name: string,
}

enum Count {
  None, Numbers, Percent
}

/**
 * Custom HTMLElement for making a table. Uses shadowDOM for cleaner HTML files.
 * Designed specifically to hold multiple badges which are dynamically added and removed.
 */
class CategoryInformation<K extends Badge> extends HTMLElement {
  #data?: CategoryData;
  /** Data stored about the element. */
  set data(data: CategoryData | undefined) { this.#data = Object.freeze(data); }
  get data() { return this.#data; }

  /** Whether to display the count of completed vs total in the header or not. */
  set count(v) { this.#count = v; }
  get count() { return this.#count; }
  #count: Count = Count.Numbers;

  /// Contains quick references to different children for global use.
  #shadow?: ShadowRoot;
  #table?: HTMLTableElement;
  #header?: HTMLSpanElement;
  badges?: Map<number, BadgeInformation<K>>;
  #style?: HTMLLinkElement;

  #badgeToProcess?: (UIBadgeData<K> | BadgeInformation<K>)[];

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
    this.#style.href = "css/tables/table.css";
    this.#style.rel = "stylesheet";

    // set header
    this.#header.title = this.#data?.name || "";
    this.#header.innerText = this.#data?.name || "";

    if (this.#badgeToProcess) this.addBadges(...this.#badgeToProcess);
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
    if (!this.#header || !this.#data || !this.badges) return;

    const completed_count = Array.from(this.badges.values()).filter(x => x.isCompleted()).length;
    const count_data = this.#countString(completed_count, this.badges?.size);
    this.#header.innerText = `${this.#data.name}${count_data}`;
  }

  /**
   * Add a badge for this element to take care of. Can take raw badge data or modified information data.
   *
   * Note: Badges can be pre-loaded, we just wait for the main element to add to the document before doing stuff with them though...
   * @param badges Information about badges to add. Can take an array or just one.
   */
  addBadges(...badges: (UIBadgeData<K> | BadgeInformation<K>)[]) {
    if (!this.#table) {
      // store badges for processing later once we get around to adding the element.
      this.#badgeToProcess = badges;
      return;
    }

    badges.forEach((badge) => {
      let row: BadgeInformation<K>;

      if ((badge as BadgeInformation<K>).data) {
        row = badge as BadgeInformation<K>;
      } else {
        row = document.createElement("badge-info") as BadgeInformation<K>;
        row.data = badge as UIBadgeData<K>;
      }

      // add to main table and storage.
      this.#table!.appendChild(row);
      this.badges!.set((row.data as UIBadgeData<K>).id, row);
    });

    this.#updateCount();
  }

  /**
   * Removes a badge this element is taking care of.
   * @param badgeId The badge to remove.
   * @returns The raw data for that badge or `undefined` if this element isn't taking care of that badge.
   */
  removeBadges(...badgeIds: number[]) {
    let badges: BadgeInformation<K>[] = [];

    badgeIds.forEach((badgeId) => {
      // attempts to get the badge and delete it.
      const entry = this.badges?.get(badgeId);
      if (!this.badges?.delete(badgeId)) return;

      // If we have deleted it succesffully, then we know that we can remove it. and return it.
      this.#table?.removeChild(entry!);
      badges.push(entry!);
    });

    return badges;
  }

  /**
   * Removes all the data ready for pre-loading.
   * @returns The data stored by doing `addBadges` when `Table` is undefined.
   */
  removePreLoaded() {
    const data = this.#badgeToProcess;
    this.#badgeToProcess = [];
    return data;
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
    this.#style.href = "css/tables/row.css";
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
   * Returns if a badge is completed or not.
   * @returns Is the badge completed. Or false if no data.
   */
  isCompleted() {
    if (!this.#data) return false;
    return this.#data?.completed > 0;
  }
}


customElements.define("category-info", CategoryInformation);
customElements.define("badge-info", BadgeInformation);
/**
 * A function which generates random testing data.
 */
function random_data(): CategoryData {
  const names = ["Forest Path", "Desert Storm", "Mountain Peak", "Ocean Waves", "City Center"];
  const towerTypes = ["Archer", "Cannon", "Magic", "Ice", "Fire", "Lightning", "Earth"];

  const randomName = names[Math.floor(Math.random() * names.length)];
  const badgeCount = Math.floor(Math.random() * 5) + 1; // Random number of badges between 1 and 5
  const randomBadges = Array.from({ length: badgeCount }, () => {
    const towerName = towerTypes[Math.floor(Math.random() * towerTypes.length)];
    const id = Math.floor(Math.random() * 1000);
    const completed = Math.random() < 0.7 ? Date.now() - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000) : 0;

    return {
      name: (hover: boolean) => towerName + (hover ? " (Hovered)" : ""),
      information: (hover: boolean) => `Information about ${towerName} ` + (hover ? " (Hovered)" : ""),
      url: `https://example.com/${towerName.toLowerCase()}`,
      id: id,
      completed: completed,
    };
  });

  return {
    name: randomName,
  };
}

const createCI = () => {
  console.log('creating new element');
  const ci = document.createElement('category-info') as CategoryInformation<Badge>;
  const data = random_data();
  ci.data = data;
  ci.count = Math.random() >
    0.33 ? Count.None : (Math.random() > 0.66 ? Count.Numbers : Count.Percent);

  const badgeCount = Math.floor(Math.random() * 5) + 1;
  const badges = Array.from({ length: badgeCount }, () => {
    const id = Math.floor(Math.random() * 1000);
    return {
      name: (hover: boolean) => `Badge ${id}` + (hover ? " (Hovered)" : ""),
      information: (hover: boolean) => `Information about badge ${id}` + (hover ? " (Hovered)" : ""),
      url: `https://example.com/badge/${id}`,
      id: id,
      completed: Math.random() < 0.7 ? Date.now() - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000) : 0,
    };
  });

  // ci.connectedCallback(); // Manually call connectedCallback

  ci.addBadges(...badges);
  document.body.appendChild(ci);
}

document.addEventListener('DOMContentLoaded', () => {
  document.getElementById("e")?.addEventListener('click', createCI);
});

for (let i = 0; i < 2; i++) {
  createCI();
}
