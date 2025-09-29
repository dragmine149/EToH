import { Badge, Lock } from "./BadgeManager";
import { noSyncTryCatch } from "./utils";

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
  /** Overview of why this is locked */
  lock_type: K['lock_type'],
  /** Indept reason as to why this is locked */
  lock_reason: K['lock_reason'],
}

interface CategoryData {
  /** Name of category */
  name: string,
  /** Overview of why this is locked */
  lock_type: Lock,
  /** Indept reason as to why this is locked */
  lock_reason?: string,
}

enum Count {
  None, Numbers, Percent
}

/**
 * Highlights a span by creating span children. Uses `innerText` to avoid having to reget the text or do weird stuff.
 * @param span The span to affect.
 * @param text The text to highlight.
 * @param selected To include the optional class for selectedness.
 */
function highlight_span(span: HTMLSpanElement, text: string, selected: boolean) {
  let regex = new RegExp(`[${text}]`, `gi`);
  span.innerHTML = span.innerText.replaceAll(regex, (match) => {
    return `<span class="highlight ${selected ? "selected" : ""}" style="margin: 0;">${match}</span>`;
  })
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
  set count(v) { this.#count = v; this.#updateCount(); }
  get count() { return this.#count; }
  #count: Count = Count.Numbers;

  #subCategories: CategoryInformation<K>;

  /// Contains quick references to different children for global use.
  #shadow?: ShadowRoot;
  #table?: HTMLTableElement;
  #gap: HTMLTableRowElement;
  #header?: HTMLSpanElement;
  #style?: HTMLLinkElement;

  /**
   * A list of badges this category is in control of. Returns data depending on it's children instead of storing stuff
   * locally.
   *
   * This does mean, every call to this getter will have to check every single child.
   */
  get badges(): Map<number, BadgeInformation<K>> {
    if (this.#table == undefined) return new Map();

    let map = new Map<number, BadgeInformation<K>>();
    for (let i = 0; i < this.#table.children.length; i++) {
      const element = this.#table.children[i];
      if (!(element instanceof BadgeInformation)) continue;

      const badge = element as BadgeInformation<K>;
      if (!badge.data) continue;

      map.set(badge.data.id, badge);
    }
    return map;
  }

  #badgeToProcess?: (UIBadgeData<K> | BadgeInformation<K>)[];

  constructor() { super(); }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { console.log('e'); }

  connectedCallback() {
    // console.log(this.clientWidth);
    // make the base required data.
    this.#shadow = this.attachShadow({ mode: "open" });
    this.#table = document.createElement("table");
    this.#gap = document.createElement("tr");
    this.#header = document.createElement("span");
    this.#style = document.createElement("link");

    // sort out shadow children
    this.#shadow.appendChild(this.#style);
    this.#shadow.appendChild(this.#header);
    this.#shadow.appendChild(this.#table);
    this.#table.appendChild(this.#gap); // random span for gap reason.

    // sort out styles
    this.classList.add("area");
    this.#style.href = "css/tables/table.css";
    this.#style.rel = "stylesheet";

    // set header
    if (this.#header) this.#header.title = this.#data?.name || "";
    if (this.#header) this.#header.innerText = this.#data?.name || "";

    // process those waiting, if we have any waiting.
    if (this.#badgeToProcess) this.addBadges(...this.#badgeToProcess);

    // Sorts out table, then sort it out again once we have style.
    // This gets around the network issue causing all those before the style has loaded once to break.
    this.#autoHide();
    if (!this.#style.sheet) this.#style.onload = this.#autoHide.bind(this);
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
   * Automatically hide this element if there is no data in the table.
   */
  #autoHide() {
    this.updateSize();

    if (this.#table == undefined) {
      this.hidden = true;
      return;
    }
    // Use <= 1 due to the invisible `gap` 1px row.
    this.hidden = this.#table?.children.length <= 1;
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
    });

    this.#autoHide();
    this.#updateCount();
  }

  addCategory(category: CategoryInformation<K>) {

  }

  /**
   * Removes a badge this element is taking care of.
   * @param badgeId The badge to remove.
   * @returns The raw data for that badge or `undefined` if this element isn't taking care of that badge.
   */
  removeBadges(...badgeIds: number[]) {
    let badges: BadgeInformation<K>[] = [];
    let stored_badges = this.badges;

    badgeIds.forEach((badgeId) => {
      // attempts to get the badge and delete it.
      const entry = stored_badges.get(badgeId);
      if (entry == undefined) return;

      // If we have deleted it succesffully, then we know that we can remove it. and return it.
      const result = noSyncTryCatch(() => this.#table?.removeChild(entry));
      if (result.error) return;
      badges.push(entry!);
    });

    this.#autoHide();
    return badges;
  }

  /** @param badgeIds The badges to show. */
  showBadges = (...badgeIds: number[]) => this.toggleBadgesVisibility(true, ...badgeIds);
  /** @param badgeIds The badges to hide. */
  hideBadges = (...badgeIds: number[]) => this.toggleBadgesVisibility(false, ...badgeIds);

  /**
   * Makes a set of badges visible / hidden. This is different to `add/remove Badges` as we keep the ownership of said badge.
   *
   * No dedicated function to a certain category as we don't know much about what to hide / not to hide.
   * @param visible To make them visible or hidden.
   * @param badgeIds The badges to affect.
   */
  toggleBadgesVisibility(visible: boolean, ...badgeIds: number[]) {
    let stored_badges = this.badges;
    badgeIds.forEach((badgeId) => {
      const entry = stored_badges.get(badgeId);
      if (entry == undefined) return;

      entry.hidden = !visible;
    });

    this.#autoHide();
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

  /**
   * Searches all the badges and highlights certain stuff depending on stuff.
   * @param data The data to search for.
   * @param is_acro Is the data an acrynoim
   */
  search(data: string, is_acro: boolean) {
    // this.badges?.forEach((b) => b.search(data, is_acro));

    if (!this.#header) return;
    highlight_span(this.#header, data, false);
  }

  /**
   * Update the size of this element according to the children element sizes.
   *
   * Allows for all children to be the same size so we have no weirdness with jumping.
   */
  updateSize() {
    let max = 0;

    // for each child.
    this.badges.forEach((b) => {
      // "get" the width and max it.
      let new_width = b.setWidth();
      max = Math.max(new_width, max);
    });

    // Then only if it's bigger than our current width. Do we set the children width. `+4` is for the table offset.
    if (max > this.clientWidth) this.style.width = `${max + 4}px`;
  }
}

/**
 * UI Element for each individual badge displayed. Aka CategoryInformation for Badges.
 */
class BadgeInformation<K extends Badge> extends HTMLElement {
  #data?: UIBadgeData<K>;
  /** Data stored about the element. */
  set data(data: UIBadgeData<K> | undefined) {
    this.#data = Object.freeze(data);
    this.#updateRow();
  }
  get data() { return this.#data; }

  /// Contains quick references to different children for global use.
  #row: HTMLTableRowElement;
  #name_field: HTMLTableCellElement;
  #info_field: HTMLTableCellElement;
  #info_data: HTMLSpanElement;
  #info_br: HTMLBRElement;
  #info_comp: HTMLSpanElement;

  constructor() {
    super();
    // To prevent duplicate children. The main nodes are only created once, in here.
    // This is extremely important as we do "remove" and "add" these elements a lot.

    this.#row = document.createElement("tr");
    this.#name_field = document.createElement("td");
    this.#info_field = document.createElement("td");
    this.#info_data = document.createElement("span");
    this.#info_br = document.createElement("br");
    this.#info_comp = document.createElement("span");

    // sort out normal children.
    this.#row.appendChild(this.#name_field);
    this.#row.appendChild(this.#info_field);
    this.#info_field.appendChild(this.#info_data);
    this.#info_field.appendChild(this.#info_br);
    this.#info_field.appendChild(this.#info_comp);
  }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { }

  connectedCallback() {
    // On update stuff to keep everything in check.
    this.appendChild(this.#row);
    this.#updateRow();
  }

  /**
   * Expands this element out to it's full form (upon hovering), set the width, collapse and return.
   *
   * This allows the parent CategoryInformation to update accordingly. Expanded is need as hover is normally longer than
   * no hover.
   * @returns The new width set.
   */
  setWidth() {
    this.#effectElement(true);
    // console.log(this.clientWidth);
    let width = this.clientWidth;
    if (this.clientWidth > 0) this.style.width = `${width}px`;
    this.#effectElement(false);
    return width;
  }

  /**
   * Update element (and over stuff eventually) when we hover.
   * @param hover Is user hover?
   */
  #effectElement(hover: boolean) {
    this.#name_field.innerText = this.#data!.name(hover);
    this.#info_data.innerText = this.#data!.information(hover);
  }

  /**
   * Update row information of the badge. (and a lot of related stuff)
   */
  #updateRow() {
    if (!this.#data) return;

    // Set the class and title for why this badge has been locked.
    switch (this.#data.lock_type) {
      case Lock.Another:
        this.classList.add("locked_another");
        this.title = `Requires ${this.#data.lock_reason} to get this badge`;
        break;
      case Lock.Temporary:
        this.classList.add("locked_temporary");
        this.title = `This badge was part of the temporary event ${this.#data.lock_reason} and is no longer obtainable.`;
        break;
      default: break;
    }

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
    return this.#data.completed > 0;
  }

  // search(data: string, is_acro: string);
}

customElements.define("category-info", CategoryInformation);
customElements.define("badge-info", BadgeInformation);


export { BadgeInformation, CategoryInformation };


// NOTE: Proof of concept.
function search(data: string) {
  let is_acro = data.startsWith("To") || data.startsWith("Co");
  let thirdLetter = data.charAt(2);
  is_acro == is_acro &&
    thirdLetter === thirdLetter.toUpperCase() && thirdLetter !== thirdLetter.toLowerCase();

  elms.forEach((elm) => elm.search(data, is_acro));
}
globalThis.search = search;


///// EVERYTHING BELOW THIS POINT IS TEST DATA TO BE DELETED AT SOME POINT.


/**
* Returns the shortened version of the text, in accordance to tower format.
* @param text The text to shorten.
*/
function shortTowerName(tower_name: string) {
  // Tower codes are made up of:
  // each word
  return tower_name.split(/[\s-]/gm)
    // lowered
    .map(word => word.toLowerCase())
    // for 'of' and 'and' to be lower, and the rest upper.
    .map(word => (word == 'of' || word == 'and') ? word[0] : word[0].toUpperCase())
    // and combined.
    .join('');
}


/**
 * A function which generates random category data.
 */
function random_Category(): CategoryData {
  const names = ["Forest Path", "Desert Storm", "Mountain Peak", "Ocean Waves", "City Center"];
  const randomName = names[Math.floor(Math.random() * names.length)];
  const locks: Lock[] = [Lock.Unlocked, Lock.Temporary, Lock.Another];
  const randomLock = locks[Math.floor(Math.random() * locks.length)];

  return {
    name: randomName,
    lock_type: randomLock,
  };
}

/**
 * A function which generates random badge data.
 */
function random_badges(): UIBadgeData<Badge>[] {
  const wordList = ["Forest", "Desert", "Mountain", "Ocean", "City", "Ancient", "Lost", "Forgotten", "Shadow", "Crystal", "Iron", "Steel", "Stone", "Fire", "Ice", "Wind", "Water", "Earth", "Sky", "Void", "and"];
  const badgeCount = Math.floor(Math.random() * 10) + 3; // Random number of badges between 1 and 5
  const locks: Lock[] = [Lock.Unlocked, Lock.Temporary, Lock.Another];

  return Array.from({ length: badgeCount }, () => {
    const wordCount = Math.floor(Math.random() * 4) + 1; // Random number of words between 1 and 4
    const towerNameWords = Array.from({ length: wordCount }, () => wordList[Math.floor(Math.random() * wordList.length)]);
    const towerName = towerNameWords.join(" ");
    const id = Math.floor(Math.random() * 1000);
    const completed = Math.random() < 0.7 ? Date.now() - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000) : 0;
    const lock_type = locks[Math.floor(Math.random() * locks.length)];

    return {
      name: (hover: boolean) => hover ? `Tower of ${towerName}` : shortTowerName(`Tower of ${towerName}`),
      information: (hover: boolean) => `Information about Tower of` + (hover ? ` ${towerName} (Hovered)` : ` ${shortTowerName(towerName)}`),
      url: `https://example.com/${towerName.toLowerCase().replace(" ", "_")}`,
      id: id,
      completed: completed,
      lock_type: lock_type,
      lock_reason: `Locked because reasons.`,
    };
  });
}

let elms: CategoryInformation<Badge>[] = [];

const createCI = () => {
  console.log('creating new element');
  const ci = document.createElement('category-info') as CategoryInformation<Badge>;
  const data = random_Category();
  ci.data = data;
  ci.count = Math.random() >
    0.33 ? Count.None : (Math.random() > 0.66 ? Count.Numbers : Count.Percent);

  ci.addBadges(...random_badges());
  document.body.appendChild(ci);
  elms.push(ci);
}

document.addEventListener('DOMContentLoaded', () => {
  document.getElementById("e")?.addEventListener('click', createCI);
  for (let i = 0; i < 2; i++) {
    createCI();
  }
});
