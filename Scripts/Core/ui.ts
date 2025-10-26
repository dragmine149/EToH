import { Badge, Lock } from "./BadgeManager";
import { loopClamp, noSyncTryCatch } from "../utils";


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

function localStorageCount() {
  const local_value = localStorage.getItem("etoh-count");
  if (local_value == null) return Count.None;
  return Number.parseInt(local_value) as Count;
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


class CategoryInformation<K extends Badge> extends HTMLElement {
  #shadow?: ShadowRoot;
  #style: HTMLLinkElement;

  #header: HTMLDivElement;
  #headerIcon: HTMLImageElement;
  #headerText: HTMLSpanElement;
  #headerSwitch: HTMLButtonElement;

  #subCategoryDiv: HTMLDivElement;
  #subCategories: SubCategoryInformation<K>[];
  #captures: CategoryInformation<K>[];

  captured: boolean;

  #category_index = 0;
  set category_index(v: number) {
    // basic loop-around clamp.
    this.#category_index = loopClamp(v, this.#subCategories.length);
    this.changeCategory();
  }
  get category_index() { return this.#category_index; }

  get #sub_category() { return this.#subCategories[this.category_index]; }
  get category_name() { return this.#sub_category.category_name; }
  get category_icon() { return this.#sub_category.icon; }
  get addBadges() { return this.#sub_category.addBadges.bind(this.#sub_category); }
  get removeBadges() { return this.#sub_category.removeBadges.bind(this.#sub_category) };
  get badges() { return this.#sub_category.badges; }

  // this lot is related to THIS object, hence why [0]
  static get observedAttributes() {
    return ['name', 'locked', 'locked_reason', 'icon'];
  }
  get name() { return this.#subCategories[0].category_name; }
  set name(v: string) { this.#subCategories[0].category_name = v; }
  get locked() { return this.#subCategories[0].locked; }
  set locked(v: Lock) { this.#subCategories[0].locked = v; }
  get locked_reason() { return this.#subCategories[0].locked_reason; }
  set locked_reason(v: string | undefined) { this.#subCategories[0].locked_reason = v; }
  get icon() { return this.#subCategories[0].icon; }
  set icon(v: string | undefined) { this.#subCategories[0].icon = v; }

  constructor() {
    super();
    console.log('e');

    this.#header = document.createElement("div");
    this.#headerIcon = document.createElement("img");
    this.#headerIcon.onerror = () => this.#headerIcon.src = "Assets/Emblems/Unknown.webp";
    this.#headerText = document.createElement("span");
    this.#headerSwitch = document.createElement("button");
    this.#headerSwitch.addEventListener('click', () => {
      this.category_index += 1;
    })

    this.#style = document.createElement("link");
    this.#style.rel = "stylesheet";
    this.#style.href = "css/category_tables.css";

    this.#header.appendChild(this.#headerIcon);
    this.#header.appendChild(this.#headerText);
    this.#header.appendChild(this.#headerSwitch);
    this.#header.id = "header";

    this.#subCategoryDiv = document.createElement("div");
    this.#subCategoryDiv.id = "sub";
    const default_sub = new SubCategoryInformation<K>();
    this.#subCategories = [default_sub];
    this.#subCategoryDiv.appendChild(default_sub.table);
    this.#captures = [];
  }

  connectedCallback() {
    this.#shadow = this.attachShadow({ mode: "open" });
    this.#shadow.appendChild(this.#style);
    this.#shadow.appendChild(this.#header);
    this.#shadow.appendChild(this.#subCategoryDiv);

    this.setMinSize();
  }

  capture(category?: CategoryInformation<K>) {
    if (category == undefined) {
      this.parentElement?.removeChild(this);
      this.captured = true;
      return this.#subCategories[0];
    }

    // undefined is only return if category is not defined so its fine.
    const sub = category.capture()!;

    this.#subCategories.push(sub);
    this.#captures.push(category);
    this.#subCategoryDiv.appendChild(sub.table);
    this.setMinSize();
    this.changeCategory(this.#subCategories.length - 1);
  }
  get captures() { return this.#captures; }

  release(category?: CategoryInformation<K>) {
    if (category == undefined) {
      this.captured = false;
      return;
    }

    const index = this.#subCategories.findIndex((sub) => sub.category_name == category.name);
    this.#subCategories.splice(index, 1);
    this.#captures.splice(index, 1);
    this.setMinSize();
    this.changeCategory();
    category.release();
    return;
  }

  changeCategory(index?: number) {
    this.#headerSwitch.hidden = this.#subCategories.length <= 1;
    // this.#sub_category.hidden = true;
    this.#subCategories.forEach((sub) => sub.hidden = true);
    if (index == undefined) index = this.category_index;
    if (index != this.#category_index) this.#category_index = loopClamp(index, this.#subCategories.length);

    if (!this.#headerSwitch.hidden) {
      const next = loopClamp(this.category_index + 1, this.#subCategories.length);
      // console.log(next);
      this.#headerSwitch.innerText = `View ${this.#subCategories[next].category_name}`;
    }

    this.#headerIcon.src = this.category_icon || "Assets/Emblems/Unknown.webp";
    this.#headerText.innerText = this.category_name;
    this.#headerText.title = `${this.category_name}${this.#sub_category.getCountString()}`;
    this.#headerText.classList[this.#sub_category.isCompleted() ? 'add' : 'remove']("rainbow");
    this.#sub_category.hidden = false;
  }

  setMinSize() {
    if (!this.#shadow) return;
    this.#subCategoryDiv.classList.remove('sized');
    this.style.width = ``;
    // console.log(this.#subCategories);
    this.#subCategories.forEach((sub) => sub.hidden = false);
    const final_size = this.#subCategories
      .map((c) => c.size)
      .reduce((m, s) => Math.max(m, s), 0) + 50;
    if (final_size > 0) this.style.width = `${final_size}px`;
    this.#subCategories.forEach((sub) => sub.hidden = true);
    this.changeCategory();
    this.#subCategoryDiv.classList.add('sized');
    console.log(final_size);
  }
}

class SubCategoryInformation<K extends Badge> {
  // category_name: string;
  locked: Lock;
  locked_reason?: string;
  icon?: string;

  set category_name(v: string) { this.table.setAttribute('name', v); }
  get category_name() { return this.table.getAttribute('name') || ""; }

  set hidden(v: boolean) { this.table.hidden = v; }
  get hidden() { return this.table.hidden; }

  /**
   * A list of badges this category is in control of. Returns data depending on it's children instead of storing stuff
   * locally.
   *
   * Every call to this getter checks all the top-level children in the table.
   */
  get badges(): Map<number, BadgeInformation<K>> {
    const map = new Map<number, BadgeInformation<K>>();
    if (this.table == undefined) return map;

    for (const element of this.table.children) {
      if (!(element instanceof BadgeInformation)) continue;

      const badge = element as BadgeInformation<K>;
      if (!badge.data) continue;

      map.set(badge.data.id, badge);
    }
    return map;
  }

  get completed(): BadgeInformation<K>[] {
    return Array.from(this.table.children)
      .filter((b) => b instanceof BadgeInformation)
      .filter((b) => b.isCompleted())
      .filter((b) => !b.hidden);
  }

  get total(): BadgeInformation<K>[] {
    return Array.from(this.table.children)
      .filter((b) => b instanceof BadgeInformation)
      .filter((b) => !b.isCompleted())
      .filter((b) => !b.hidden);
  }

  get size() {
    const size = Array.from(this.table.children)
      .filter((child) => child instanceof BadgeInformation)
      .map((child) => child.setWidth())
      .map((numbers) => numbers
        .reduce((m, s) => m + s, 0))
      .reduce((m, s) => Math.max(m, s), 0);
    return Math.ceil(size / 100) * 100;
  }

  table: HTMLTableElement;
  #gap: HTMLTableRowElement;

  constructor() {
    this.table = document.createElement("table");
    this.#gap = document.createElement("tr");
    this.table.appendChild(this.#gap);
  }

  /**
   * Add a badge for this element to take care of. Can take raw badge data or modified information data.
   *
   * @param badges Information about badges to add. Can take an array or just one.
   */
  addBadges(...badges: (BadgeInformation<K> | UIBadgeData<K>)[]) {
    badges.forEach((badge) => {
      // we can already use as-is
      if ((badge as BadgeInformation<K>).data) {
        this.table.appendChild(badge as BadgeInformation<K>);
        return;
      }

      // but we might have to translate
      const row = document.createElement("badge-info") as BadgeInformation<K>;
      row.data = badge as UIBadgeData<K>;
      this.table.appendChild(row);
    })
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
      const result = noSyncTryCatch(() => this.table.removeChild(entry));
      if (result.error) return;
      badges.push(entry);
    });

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
    const stored_badges = this.badges;
    badgeIds.forEach((badgeId) => {
      const entry = stored_badges.get(badgeId);
      if (entry == undefined) return;

      entry.hidden = !visible;
    });
  }

  getCountString(count_type?: Count) {
    if (count_type == undefined) count_type = localStorageCount();

    switch (count_type) {
      case Count.None: return ``;
      case Count.Numbers: return ` (${this.completed.length}/${this.total.length})`;
      case Count.Percent: return ` (${this.total.length === 0 ? 0 : ((this.completed.length / this.total.length) * 100).toFixed(2)})`;
    }
  }

  isCompleted() { return this.completed.length == this.total.length; }
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
  // #row: HTMLTableRowElement;
  #name_field: HTMLTableCellElement;
  #info_field: HTMLTableCellElement;
  #info_data: HTMLSpanElement;
  #info_br: HTMLBRElement;
  #info_comp: HTMLSpanElement;

  constructor() {
    super();
    // To prevent duplicate children. The main nodes are only created once, in here.
    // This is extremely important as we do "remove" and "add" these elements a lot.

    // this.#row = document.createElement("tr");
    this.#name_field = document.createElement("td");
    this.#info_field = document.createElement("td");
    this.#info_data = document.createElement("span");
    this.#info_br = document.createElement("br");
    this.#info_comp = document.createElement("span");

    // sort out normal children.
    // this.#row.appendChild(this.#name_field);
    // this.#row.appendChild(this.#info_field);
    this.#info_field.appendChild(this.#info_data);
    this.#info_field.appendChild(this.#info_br);
    this.#info_field.appendChild(this.#info_comp);
  }
  // This is empty because we don't want to recreate a ton of stuff.
  connectedMoveCallback() { return; }

  connectedCallback() {
    // On update stuff to keep everything in check.
    // this.appendChild(this.#row);
    this.appendChild(this.#name_field);
    this.appendChild(this.#info_field);
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
    this.onmouseover = this.#effectElement.bind(this, true);
    this.onmouseleave = this.#effectElement.bind(this, false);

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

if (customElements.get('category-info') == undefined) customElements.define("category-info", CategoryInformation);
if (customElements.get('badge-info') == undefined) customElements.define("badge-info", BadgeInformation);


export { BadgeInformation, UIBadgeData, CategoryData, Count, BadgeUserData, localStorageCount, CategoryInformation };
