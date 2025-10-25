import { loopClamp, noSyncTryCatch } from "../utils";
import { Badge, Lock } from "./BadgeManager";
import { BadgeInformation, Count, localStorageCount, UIBadgeData } from "./ui";

class CategoryInformation<K extends Badge> extends HTMLElement {
  #shadow?: ShadowRoot;
  #style: HTMLLinkElement;

  #header: HTMLDivElement;
  #headerIcon: HTMLImageElement;
  #headerText: HTMLSpanElement;
  #headerSwitch: HTMLDivElement;

  #subCategoryDiv: HTMLDivElement;
  #subCategories: SubCategoryInformation<K>[];
  #captures: CategoryInformation<K>[];

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
    this.#headerSwitch = document.createElement("div");
    this.#headerSwitch.click = () => this.category_index += 1;

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
    if (category == undefined) throw new Error("help");

    const index = this.#subCategories.findIndex((sub) => sub.category_name == category.name);
    this.#subCategories.splice(index, 1);
    this.#captures.splice(index, 1);
    this.setMinSize();
    this.changeCategory();
    return;
  }

  changeCategory(index?: number) {
    this.#sub_category.hidden = true;
    if (index == undefined) index = this.category_index;
    if (index != this.#category_index) this.#category_index = loopClamp(index, this.#subCategories.length);

    this.#headerIcon.src = this.category_icon || "Assets/Emblems/Unknown.webp";
    this.#headerText.innerText = this.category_name;
    this.#headerText.title = `${this.category_name}${this.#sub_category.getCountString()}`;
    this.#headerText.classList[this.#sub_category.isCompleted() ? 'add' : 'remove']("rainbow");
    this.#sub_category.hidden = false;
  }

  setMinSize() {
    if (!this.#shadow) return;
    this.style.width = ``;
    // console.log(this.#subCategories);
    this.#subCategories.forEach((sub) => sub.hidden = false);
    const final_size = this.#subCategories
      .map((c) => c.size)
      .reduce((m, s) => Math.max(m, s), 0) + 50;
    if (final_size > 0) this.style.width = `${final_size}px`;
    this.#subCategories.forEach((sub) => sub.hidden = true);
    this.changeCategory();
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

export { CategoryInformation };
