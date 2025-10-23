import { Badge, Lock } from "../ETOHBridge/BadgeManager";
import { noSyncTryCatch } from "../utils";

interface UIBadgeData<K extends Badge> {
  /** Function to call to show information in the name field of the ui. */
  name: K['get_name_field'],
  /** Function to call to set the style of the element. Shouldn't be used often */
  name_style: K['set_name_style'],
  /** Function to call to show information in the info field of the ui. */
  information: K['get_information_field'],
  /** Function to call to set the style of the element. Shouldn't be used often */
  info_style: K['set_info_style'],
  /** Link to the badge on roblox itself. */
  url: K['link'],
  /** Badge ID */
  id: K['id'],
  /** Link to a wiki page about said badge. */
  wiki?: URL,
  /** Overview of why this is locked */
  lock_type: K['lock_type'],
  /** Indept reason as to why this is locked */
  lock_reason: K['lock_reason'],
}

interface BadgeUserData {
  /** Completed date in utc time (via `new Date().getTime()`) */
  completed: number,
}

interface CategoryData {
  /** Name of category */
  name: string,
  /** Overview of why this is locked */
  lock_type: Lock,
  /** Indept reason as to why this is locked */
  lock_reason?: string,
  /** Path to the icon to display on the left side of the dropdown.. */
  icon?: string,
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
  const regex = new RegExp(`[${text}]`, `gi`);
  span.innerHTML = span.innerText.replaceAll(regex, (match) => {
    return `<span class="highlight ${selected ? "selected" : ""}" style="margin: 0;">${match}</span>`;
  })
}

function createArrowSVG(direction: 'up' | 'down' = 'down', className?: string): SVGSVGElement {
  const svgNS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(svgNS, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("xmlns", svgNS);
  if (className) svg.setAttribute("class", className);

  const path = document.createElementNS(svgNS, "path");
  path.setAttribute("fill-rule", "evenodd");
  path.setAttribute("clip-rule", "evenodd");

  const DOWN_D = "M4.29289 8.29289C4.68342 7.90237 5.31658 7.90237 5.70711 8.29289L12 14.5858L18.2929 8.29289C18.6834 7.90237 19.3166 7.90237 19.7071 8.29289C20.0976 8.68342 20.0976 9.31658 19.7071 9.70711L12.7071 16.7071C12.3166 17.0976 11.6834 17.0976 11.2929 16.7071L4.29289 9.70711C3.90237 9.31658 3.90237 8.68342 4.29289 8.29289Z";
  const UP_D = "M12 7C12.2652 7 12.5196 7.10536 12.7071 7.29289L19.7071 14.2929C20.0976 14.6834 20.0976 15.3166 19.7071 15.7071C19.3166 16.0976 18.6834 16.0976 18.2929 15.7071L12 9.41421L5.70711 15.7071C5.31658 16.0976 4.68342 16.0976 4.29289 15.7071C3.90237 15.3166 3.90237 14.6834 4.29289 14.2929L11.2929 7.29289C11.4804 7.10536 11.7348 7 12 7Z";

  path.setAttribute("d", direction === "down" ? DOWN_D : UP_D);
  svg.appendChild(path);

  return svg;
}

/**
 * Custom HTMLElement for making a table. Uses shadowDOM for cleaner HTML files.
 * Designed specifically to hold multiple badges which are dynamically added and removed.
 */
class CategoryInformation<K extends Badge> extends HTMLElement {
  // ====================================================================================================
  // This section is for data which can be affected externally by the user.
  // ====================================================================================================

  #data?: CategoryData;
  /** Data stored about the element. */
  set data(data: CategoryData | undefined) { this.#data = Object.freeze(data); this.#updateData(); }
  get data() { return this.#data; }

  /** Whether to display the count of completed vs total in the header or not. */
  set count(v) { this.#count = v; this.#updateCount(); }
  get count() { return this.#count; }
  #count: Count = Count.Numbers;

  /**
   * A list of badges this category is in control of. Returns data depending on it's children instead of storing stuff
   * locally.
   *
   * Every call to this getter checks all the top-level children in the table.
   */
  get badges(): Map<number, BadgeInformation<K>> {
    const map = new Map<number, BadgeInformation<K>>();
    if (this.#table == undefined) return map;

    for (const element of this.#table.children) {
      if (!(element instanceof BadgeInformation)) continue;

      const badge = element as BadgeInformation<K>;
      if (!badge.data) continue;

      map.set(badge.data.id, badge);
    }
    return map;
  }

  /**
   * A list of sub-categories this category is in control of. Returns data depending on children instead of local storage
   * guessing.
   *
   * Every call to this getter checks all the top-level children in the shadow.
   */
  get categories(): CategoryInformation<K>[] {
    if (this.#shadow == undefined) return [];
    // console.log('getting categories', this.#shadow.children);
    return Array.from(this.#shadow.children).filter((child) => child instanceof CategoryInformation);
  }

  // ====================================================================================================
  // Now we get into the section of behind-the-scenes stuff
  // ====================================================================================================

  /** A list of categories we are still yet to process due to the shadow no being ready */
  #categoriesToProcess?: (CategoryInformation<K>)[];

  /** The main child, everything is hidden in here */
  #shadow?: ShadowRoot;
  /** Reference to the table where all the badges are displayed */
  #table: HTMLTableElement;
  /** A dummy element for a 1px extra stylelish gap between the header and the content */
  #gap: HTMLTableRowElement;
  /** The header of the table, containing the title, count/progression, and the button for sub-children */
  #header: HTMLDivElement;
  #headerIcon: HTMLImageElement;
  #headerText: HTMLSpanElement;
  #headerDrop: HTMLDivElement;
  #headerDropDown: SVGSVGElement;
  #headerDropUp: SVGSVGElement;
  /** The style element as shadows do not read style from main body by default. */
  #style: HTMLLinkElement;

  /** Internal state of the sub categories. Are they visible or not. */
  #subCategoryState = true;

  // ====================================================================================================
  // The code to run on element creation / add to DOM.
  // ====================================================================================================

  /** Sets up the basics things that can be created on creation as we can off-screen manipulate this */
  constructor() {
    super();

    this.#table = document.createElement("table");
    this.#gap = document.createElement("tr");
    this.#header = document.createElement("div");
    this.#headerIcon = document.createElement("img");
    this.#headerText = document.createElement("span");
    this.#headerDrop = document.createElement("div");
    this.#headerDropDown = createArrowSVG('down', 'hidden');
    this.#headerDropUp = createArrowSVG('up', 'hidden');
    this.#style = document.createElement("link");

    // random span for gap reason.
    this.#table.appendChild(this.#gap);

    this.#style.href = "css/tables/table.css";
    this.#style.rel = "stylesheet";

    this.#header.appendChild(this.#headerIcon);
    this.#header.appendChild(this.#headerText);
    this.#header.appendChild(this.#headerDrop);
    this.#headerDrop.appendChild(this.#headerDropDown);
    this.#headerDrop.appendChild(this.#headerDropUp);

    this.#headerDrop.addEventListener('click', () => {
      if (this.categories.length <= 0) return;

      // console.log('header click');
      this.toggleCategoryVisibility(!this.#subCategoryState);
      this.#headerDropDown.classList.toggle('hidden');
      this.#headerDropUp.classList.toggle('hidden');
    })
  }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { console.log('e'); }

  /** Add to DOM and setup the things that could not be setup previously due to various reasons. */
  connectedCallback() {
    // make the base required data.
    this.#shadow = this.attachShadow({ mode: "open" });

    // sort out shadow children
    this.#shadow.appendChild(this.#style);
    this.#shadow.appendChild(this.#header);
    this.#shadow.appendChild(this.#table);

    // sort out styles
    this.classList.add("area");

    // set header
    this.#headerText.title = this.#data?.name || "";
    this.#headerText.innerText = this.#data?.name || "";
    this.#headerIcon.src = this.#data?.icon || "Assets/Emblems/Unknown.webp";
    this.#headerIcon.onerror = () => this.#headerIcon.src = "Assets/Emblems/Unknown.webp";

    // process those waiting, if we have any waiting.
    // console.log(`Adding categories from queue`, this.#categoriesToProcess);
    if (this.#categoriesToProcess) this.addCategory(...this.#categoriesToProcess);

    // Sorts out table, then sort it out again once we have style.
    // This gets around the network issue causing all those before the style has loaded once to break.
    this.#autoHide();
    if (!this.#style.sheet) this.#style.onload = this.#autoHide.bind(this);
  }

  // ====================================================================================================
  // Now we get into back-end UI management
  // ====================================================================================================

  /**
   * Update elements that rely on the data object when they have been set.
   */
  #updateData() {
    this.#headerText.title = this.#data!.name;
    this.#headerText.innerText = this.#data!.name;
    this.#headerIcon.src = this.#data!.icon || "Assets/Emblems/Unknown.webp";
  }


  /**
   * Automatically hide this element if there is no data in the table.
   */
  #autoHide() {
    this.updateSize();

    // Use <= 1 due to the invisible `gap` 1px row.
    // also check for categories. As no badges != no categories. (Thanks Windswept Peaks)
    this.hidden = this.#table?.children.length <= 1 && this.categories.length <= 0;
  }

  /**
   * Update the size of this element according to the children element sizes.
   *
   * Allows for all children to be the same size so we have no weirdness with jumping.
   */
  updateSize() {
    const sizes: number[][] = [
      ...this.categories.map((c) => c.updateSize()),
      ...Array.from(this.badges.values()).map((b) => b.setWidth()),
    ];
    const name = sizes.map((s) => s[0]).reduce((m, s) => Math.max(m, s), 0);
    const info = sizes.map((s) => s[1]).reduce((m, s) => Math.max(m, s), 0);

    if (name + info > 0) this.style.width = `${name + info + 8}px`;
    return [name, info];
  }

  /**
   * Formats a string to display counted data.
   * @param completed The completed element count.
   * @param total The total element count.
   * @returns A formatted string based off Count enum.
   */
  #countString(completed: number, total: number) {
    // nice and simple
    if (this.count == Count.Numbers) return ` (${completed}/${total})`;
    if (this.count == Count.Percent) {
      // need to do a tad bit of maths
      const percentage = (total === 0) ? 0 : ((completed / total) * 100);
      // 2dp is perfect. No need to make setting for it either as kinda recognised everywhere.
      return ` (${percentage.toFixed(2)}%)`;
    }
    // Also accounts for Count.None
    return ``;
  }

  /**
   * Updates the count display.
   */
  #updateCount() {
    const completed_count = Array.from(this.badges.values()).filter(x => x.isCompleted()).length;
    const count_data = this.#countString(completed_count, this.badges?.size);
    this.#headerText.innerText = `${this.#data?.name || ""}${count_data}`;
    this.#headerText.classList[completed_count == this.badges.size ? 'add' : 'remove']("rainbow");
  }

  // ====================================================================================================
  // Now we deal with adding / hiding / removing / showing
  // ====================================================================================================

  /**
   * Add a badge for this element to take care of. Can take raw badge data or modified information data.
   *
   * @param badges Information about badges to add. Can take an array or just one.
   */
  addBadges(...badges: (UIBadgeData<K> | BadgeInformation<K>)[]) {
    // now we process said badges
    badges.forEach((badge) => {
      // we can already use as-is
      if ((badge as BadgeInformation<K>).data) {
        this.#table.appendChild(badge as BadgeInformation<K>);
        return;
      }

      // but we might have to translate
      const row = document.createElement("badge-info") as BadgeInformation<K>;
      row.data = badge as UIBadgeData<K>;
      this.#table.appendChild(row);
    });

    // Update the UI with the new badges.
    this.#autoHide();
    this.#updateCount();
  }

  /**
   * Add a category to act kinda like a sub-area. Useful for grouping stuff.
   *
   * Note: pre-loads the categories waiting for the element to be added to the DOM due to the use of shadow-dom.
   */
  addCategory(...categories: CategoryInformation<K>[]) {
    if (!this.#shadow) {
      this.#categoriesToProcess = [...(this.#categoriesToProcess || []), ...categories];
      return;
    }
    categories.forEach((cat) => {
      this.#shadow?.appendChild(cat);
    });

    this.#headerDropUp.classList.remove('hidden');
  }

  /**
   * Removes a badge this element is taking care of.
   * @param badgeId The badge to remove.
   * @returns The raw data for that badge or `undefined` if this element isn't taking care of that badge.
   */
  removeBadges(...badgeIds: number[]) {
    const badges: BadgeInformation<K>[] = [];
    const stored_badges = this.badges;

    badgeIds.forEach((badgeId) => {
      // attempts to get the badge and delete it.
      const entry = stored_badges.get(badgeId);
      if (entry == undefined) return;

      // If we have deleted it succesffully, then we know that we can remove it. and return it.
      const result = noSyncTryCatch(() => this.#table.removeChild(entry));
      if (result.error) return;
      badges.push(entry);
    });

    this.#autoHide();
    return badges;
  }

  /**
   * Removes a category this element is taking care of.
   * @param indexes List of indexes to remove
   * @returns The CategoryInformation for that index. (or nothing if index out of range)
   */
  removeCategory(...indexes: number[]) {
    const elms = this.categories.filter((v, i) => i in indexes);
    elms.forEach((elm) => this.#shadow?.removeChild(elm));

    if (this.categories.length <= 0) {
      this.#headerDropUp.classList.remove('hidden');
    }
    return elms;
  }

  /** @param badgeIds The badges to show. */
  showBadges = (...badgeIds: number[]) => this.toggleBadgesVisibility(true, ...badgeIds);
  /** @param badgeIds The badges to hide. */
  hideBadges = (...badgeIds: number[]) => this.toggleBadgesVisibility(false, ...badgeIds);
  /** */
  showCategories = () => this.toggleCategoryVisibility(true);
  /** */
  hideCategories = () => this.toggleCategoryVisibility(false);

  /**
   * Makes a set of badges visible / hidden. This is different to `add/remove Badges` as we keep the ownership of said badge.
   *
   * No dedicated function to a certain category as we don't know much about what to hide / not to hide.
   * @param visible To make them visible or hidden.
   * @param badgeIds The badges to affect.
   */
  toggleBadgesVisibility(visible: boolean, ...badgeIds: number[]) {
    const stored_badges = this.badges;
    badgeIds.forEach((badgeId) => {
      const entry = stored_badges.get(badgeId);
      if (entry == undefined) return;

      entry.hidden = !visible;
    });

    this.#autoHide();
  }

  /**
   * Makes all the sub-categories invisible or not.
   */
  toggleCategoryVisibility(visible: boolean) {
    this.#subCategoryState = visible;
    this.categories.forEach((cat) => cat.hidden = !visible);
    this.#autoHide();
  }

  /**
   * Removes all the data ready for pre-loading.
   * @returns The data stored by doing `addCategories` when `#Shadow` is undefined.
   */
  removeCategoriesInQueue() { return this.#categoriesToProcess?.splice(0); }
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

  /** Data stored about the specific user. This is meant to change all the time without a need to regenerate the whole thing. */
  // user_badge_data?: BadgeUserData;
  #user_data?: BadgeUserData;
  set user_badge_data(data: BadgeUserData | undefined) {
    this.#user_data = data;
    this.#updateRow();
  }
  get user_badge_data() { return this.#user_data; }

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
  connectedMoveCallback() { return; }

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
    // this.style.width = "";
    // console.log(this.clientWidth);

    const name = this.#name_field.clientWidth;
    const data = this.#info_field.clientWidth;

    // const width = this.clientWidth;
    // if (this.clientWidth > 0) this.style.width = `${width}px`;
    this.#effectElement(false);
    return [name, data];
  }

  /**
   * Update element (and over stuff eventually) when we hover.
   * @param hover Is user hover?
   */
  #effectElement(hover: boolean) {
    if (!this.#data) return;
    this.#name_field.innerHTML = this.#data.name(hover);
    this.#info_data.innerHTML = this.#data.information(hover);
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
    this.#name_field.innerHTML = this.#data.name(false);
    this.#info_data.innerHTML = this.#data.information();
    this.#info_comp.innerHTML = this.isCompleted() ? new Date(this.user_badge_data!.completed).toLocaleString(undefined, {
      year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
    }) : '';

    // sort out external events.
    this.#row.onmouseover = this.#effectElement.bind(this, true);
    this.#row.onmouseleave = this.#effectElement.bind(this, false);

    this.setNameStyle();
    this.setInfoStyle();
  }

  /**
   * Public function to update the completed date of this badge.
   * @param completed Optional completed unix timestamp (`new Date().getTime()`)
   */
  updatedCompleted(completed?: number) {
    this.#info_comp.innerHTML = completed && completed > 0 ? new Date(completed).toLocaleString(undefined, {
      year: "numeric", month: "numeric", day: "numeric", hour: "numeric", minute: "numeric", second: "numeric", hour12: false,
    }) : '';
  }

  /**
   * Returns if a badge is completed or not.
   * @returns Is the badge completed. Or false if no data.
   */
  isCompleted() {
    if (!this.user_badge_data) return false;
    return this.user_badge_data.completed > 0;
  }

  setNameStyle(style?: string) {
    this.#name_field.style = style ?? (this.#data ? this.#data.name_style(this.user_badge_data) : "");
  }
  setInfoStyle(style?: string) {
    this.#info_field.style = style ?? (this.#data ? this.#data.info_style(this.user_badge_data) : "");
  }

  // search(data: string, is_acro: string);
}

customElements.define("category-info", CategoryInformation);
customElements.define("badge-info", BadgeInformation);


export { BadgeInformation, CategoryInformation, UIBadgeData, CategoryData, Count, BadgeUserData };
