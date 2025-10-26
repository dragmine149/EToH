import { Area, areaManager } from "../ETOHBridge/AreaManager";
import { Badge } from "../Core/BadgeManager";
import { Category, Tower, userManager, badgeManager, EToHUser, Other } from "./Etoh";
import { load_required_data } from "../ETOHBridge/data_loader";
import { isMobile } from "../utils";
import { BadgeInformation, UIBadgeData, CategoryInformation } from "../Core/ui";

enum PreloadState {
  TowerData,
  OtherData,
  TowerWarning,
  OtherWarning,
  Errored,
  Finished
}

/**
 * Turns PreloadState into a string for display purposes.
 * @param state The current state of preloading.
 * @returns A string version to display as `{state}: some message`
 */
function preload_status(state: PreloadState) {
  switch (state) {
    case PreloadState.TowerData:
    case PreloadState.OtherData:
      return "Loading";
    case PreloadState.TowerWarning:
    case PreloadState.OtherWarning:
      return "WARNING"
    case PreloadState.Errored:
      return "ERROR"
    case PreloadState.Finished:
      return "Completed"
  }
}

/**
* Higher level TS class, built upon the modules of everything else.
*
* Designed to keep everything to do with navigation around the UI in one place.
*/
class UI {
  // Stuff related to loading the data at initial load.
  #preload_parent: HTMLDivElement;
  #preload: HTMLDivElement;
  #preload_span: HTMLSpanElement;
  #preload_button: HTMLButtonElement;
  #loaded: boolean;
  set loaded(v) { this.#loaded = v; this.#preload_parent.hidden = v; this.#search_main.hidden = !v; }
  get loaded() { return this.#loaded; }
  // How many times to retry before we have to say. Sorry, but it's not going to load.
  #retry_count: 0 | 1 | 2 = 0;

  // Stuff related to loading user data.
  #main_nav_toggle: HTMLInputElement;
  #main_menu_btn: HTMLAnchorElement;
  #search_main: HTMLDivElement;
  #user_list: HTMLDataListElement;

  // Stuff related to the displaying of users.
  #user: HTMLDivElement;
  // #user_img: HTMLAnchorElement;
  #user_menu_timers: number[];
  #user_img: HTMLImageElement;
  #user_menu: HTMLDivElement;
  #user_link: HTMLAnchorElement;
  #user_update: HTMLButtonElement;
  #user_default: HTMLButtonElement;
  #user_favourite: HTMLButtonElement;
  #user_delete: HTMLButtonElement;

  #user_mini_search: HTMLDivElement;
  #user_mini_input: HTMLInputElement;
  #user_search: HTMLInputElement;
  #user_search_button: HTMLButtonElement;
  #user_load_error: HTMLSpanElement;
  #user_search_back: HTMLButtonElement;
  #user_mini_button: HTMLButtonElement;
  #user_mini_viewing: HTMLSpanElement;

  // Stuff related to storing UI information
  #categories: Map<string, CategoryInformation<Badge>>;
  #badges: Map<number, BadgeInformation<Badge>>;

  // stuff related to dispaling of badges.
  #badgesUI: HTMLDivElement;

  constructor() {
    // set this straight away to show it and ignore the noscript element popup. The parent object is more important.
    this.#preload_parent = document.getElementById("pre-load") as HTMLDivElement;
    this.#preload_parent.hidden = false;

    this.#preload = document.getElementById("pre-load-info") as HTMLDivElement;
    this.#preload_span = this.#preload.children[2] as HTMLSpanElement;
    this.#preload_button = this.#preload.lastElementChild as HTMLButtonElement;
    // console.log(this.#preload);
    // console.log(this.#preload_span);
    // console.log(this.#preload_button);

    // In the rare case data is not loaded, some prevention for it.
    this.#preload_button.addEventListener('click', () => {
      this.#retry_count += 1;
      if (this.#retry_count >= 2) {
        this.#preload_button.disabled = true;
        this.#preload_span.textContent = `Failed to load required data 3 times. Please try again later with better connection or report a bug on github.`;
        return;
      }

      this.#preload.classList.remove("errored");
      this.#preload_button.hidden = true;
      load_required_data();
    })

    // basic search
    this.#main_nav_toggle = document.getElementById("nav-hide") as HTMLInputElement;
    this.#main_menu_btn = document.getElementById("main_menu_button") as HTMLAnchorElement;
    this.#main_menu_btn.onclick = () => this.main_user_load();

    this.#search_main = document.getElementById("search-main") as HTMLDivElement;
    this.#user_list = document.getElementById("user_list") as HTMLDataListElement;

    // advanced search and user profile
    this.#user = document.getElementsByTagName("user").item(0) as HTMLDivElement;
    this.#user_img = document.getElementById("user-profile") as HTMLImageElement;
    this.#user_menu = document.getElementById("usermenu") as HTMLImageElement;
    this.#user_link = this.#user_menu.firstElementChild as HTMLAnchorElement;
    this.#user_update = document.getElementById("usermenu-update") as HTMLButtonElement;
    this.#user_default = document.getElementById("usermenu-default") as HTMLButtonElement;
    this.#user_favourite = document.getElementById("usermenu-favourite") as HTMLButtonElement;
    this.#user_delete = document.getElementById("usermenu-delete") as HTMLButtonElement;

    this.#user_update.addEventListener('click', () => this.updateUserData());
    // This section makes the user menu work.
    this.#user_default.addEventListener('click', () => {
      this.#user_menu.hidden = true;
      localStorage.setItem("etoh-default", userManager.current_user!.id.toString());
    });
    this.#user_menu_timers = [0, 0, 0, 0];
    this.#user_menu.addEventListener('mouseover', () => {
      if (this.#user_menu.dataset.visible === "false") return;
      this.#userMenuHover(true)
    });
    this.#user_menu.addEventListener('mouseleave', () => this.#userMenuHover(false));
    this.#user_img.addEventListener('mouseover', () => this.#userMenuHover(true));
    this.#user_img.addEventListener('mouseleave', () => this.#userMenuHover(false));

    // and more defining of elements.
    this.#user_search = document.getElementById("search_input") as HTMLInputElement;
    this.#user_search_button = document.getElementById("search_button") as HTMLButtonElement;
    this.#user_search_back = document.getElementById("search_back") as HTMLButtonElement;
    this.#user_load_error = document.getElementById("load_errors") as HTMLSpanElement;
    this.#user_mini_search = document.getElementById("mini-search") as HTMLDivElement;
    this.#user_mini_input = this.#user_mini_search.firstElementChild as HTMLInputElement;
    this.#user_mini_button = this.#user_mini_search.lastElementChild as HTMLButtonElement;
    this.#user_mini_viewing = (document.getElementById("viewing") as HTMLDivElement).firstElementChild as HTMLSpanElement;

    // initial settings, bindings, etc that need to be made for everything to work.
    this.#user_img.hidden = true;
    this.#user_search.onkeydown = this.#submitUserSearch.bind(this);
    this.#user_search.oninput = this.#syncUserSearch.bind(this);
    this.#user_mini_input.oninput = this.#syncUserSearch.bind(this);
    this.#user_search.value = "";
    this.#user_mini_input.value = "";
    this.#user_search_button.onmousedown = this.#submitUserSearch.bind(this);
    this.#user_mini_button.onmousedown = this.#submitUserSearch.bind(this);
    this.#user_search_button.disabled = this.#user_search.value.length <= 0;
    this.#user_search_back.disabled = true;
    this.#user_search_back.onclick = () => this.#search_main.hidden = true;
    this.#user.onclick = () => this.#miniSearch(true);
    this.#user_mini_input.onclick = () => this.#miniSearch(true);
    this.#user_mini_input.onblur = () => this.#miniSearch(false);
    this.#user_mini_input.onfocus = () => this.#user_mini_input.select();
    this.#user_mini_input.onkeydown = this.#submitUserSearch.bind(this);

    this.#categories = new Map();
    this.#badges = new Map();
    this.#badgesUI = document.getElementById("badges") as HTMLDivElement;
  }

  #userMenuHover(hover: boolean) {
    if (this.#user_menu_timers[0]) {
      clearTimeout(this.#user_menu_timers[0]);
      clearTimeout(this.#user_menu_timers[1]);
      clearTimeout(this.#user_menu_timers[2]);
    }

    this.#user_menu_timers[0] = setTimeout(() => {
      if (hover) this.#user_menu.hidden = false;
      this.#user_menu_timers[1] = setTimeout(() => this.#user_menu.dataset.visible = hover.toString(), 50);
      if (!hover) this.#user_menu_timers[2] = setTimeout(() => this.#user_menu.hidden = true, 200);
    }, hover ? 0 : 300);
  }

  /**
   * Syncs the main search with the mini search and vis-versa.
   * @param ev The event. Used to get the value as we have no idea which to overwrite otherwise.
   */
  #syncUserSearch(ev: InputEvent) {
    const target = ev.target as HTMLInputElement;
    this.#user_mini_input.value = target.value;
    this.#user_search.value = target.value;
    this.#user_search_button.disabled = this.#user_search.value.length <= 0;
  }

  /**
   * User submits a search, we have to process and do stuff now.
   * @param ev The event to check for `ENTER`. Has to take other types just to make ts happy.
   */
  #submitUserSearch(ev: KeyboardEvent | MouseEvent | SubmitEvent) {
    if (ev instanceof KeyboardEvent) {
      if (ev.key !== 'Enter') return;
    }
    this.#miniSearch(false);
    this.load_user(this.#user_search.value);
  }

  /**
   * Enables / Disables the minisearch. Mobile is disabled no matter what.
   * @param enabled Should the minisearch be enablled or not.
   */
  #miniSearch(enabled: boolean) {
    if (isMobile()) return;

    this.#user.hidden = enabled;
    this.#user_mini_search.hidden = !enabled;
    this.#user_mini_viewing.textContent = enabled ? "Load user" : "Currently viewing"
    if (enabled) this.#user_mini_input.focus();
  }

  /**
   * Front-end UI for initial loading of sub-site site.
   * @param message The message to show of the current status.
   * @param state The state we are currently in.
   */
  preload(message: string, state: PreloadState) {
    // console.log(message, state);
    this.loaded = state == PreloadState.Finished;

    this.#preload_span.textContent = `${preload_status(state)}: ${message}`;
    if (state == PreloadState.Errored) {
      this.#preload_button.hidden = false;
      this.#preload.classList.add("errored");
    }
  }

  /**
   * Adds a new user to the datalist object used for the search input fields. Also prevents users from being added twice.
   * @param list The list of users.
   */
  datalist_add_user(...list: string[]) {
    const children = this.#user_list.children;
    list.filter((user) => children.namedItem(user) == null).forEach((user) => {
      console.log(`Adding ${user} to the user-list`);
      const option = document.createElement("option");
      option.textContent = user;
      option.value = user;
      option.setAttribute("name", user);
      this.#user_list.appendChild(option);
    })
  }

  #updateDefaultOption(current_user: EToHUser) {
    const stored_user = localStorage.getItem("etoh-default");
    if (stored_user == null) return;
    const is_default = current_user.id != Number.parseInt(stored_user);

    this.#user_default.disabled = !is_default;
    this.#user_default.innerText = !is_default ? "Already default user" : "Set default user";
    this.#user_default.title = !is_default ? "This user is already being loaded upon no url parameter" : "If URL user search parameter doesn't exist, load this user.";
  }

  /**
   * Loads a user via userManager. Updates the UI accordingly etc.
   *
   * There is no "unload" event as loading a new user will unload the old user.
   * @param user_input The user the user inputted.
   * @param popped Stops us from editing the history by modifying the state.
   */
  async load_user(user_input: string | number, popped?: boolean) {
    const old_user = userManager.current_user;
    const user = await userManager.find_user(user_input);
    if (user == undefined) { return; }
    userManager.store_user(old_user);
    this.datalist_add_user(user.name);

    // no await as we do in background, hopefully.
    const loading = user.loadDatabaseBadges();

    this.#user.textContent = user.ui_name;
    this.#user_link.href = user.link;
    this.#user_img.src = user.profile;

    this.#user_img.hidden = false;
    this.#user_search_back.disabled = false;
    // this style is so that its hopefully invisible when no user loaded.
    this.#user_mini_button.style.right = "3.4rem";
    this.#updateDefaultOption(user);

    const url = new URL(location.toString());
    url.searchParams.set("user", user.name);
    if (popped == undefined || popped == false) history.pushState(undefined, "", url);

    await loading;
    this.#search_main.hidden = true;
    this.updateUIData();
    await this.updateUserData();
  }

  async updateUserData() {
    const user = userManager.current_user;
    if (user == undefined) return;
    const badges = badgeManager.uncompleted(Array.from(user.completed.keys()));
    await user.loadServerBadges(badges);

    this.updateUIData();
  }

  updateUIData() {
    const user = userManager.current_user;
    if (user == undefined) return;
    // reset the user badge data as it breaks stuff otherwise.
    Array.from(this.#badges.values()).forEach((b) => b.user_badge_data = undefined);

    for (const [id, time] of user.completed) {
      const badgeUI = this.#badges.get(id);
      if (badgeUI == undefined) {
        console.warn(`Somehow user has a badge not defined, please report ${id}`);
        continue;
      }
      badgeUI.user_badge_data = {
        completed: time
      };
    }
  }

  /**
   * Loads the main menu, should be a really small function but a separate function just in case.
   */
  async main_user_load() {
    this.#search_main.hidden = false;
    this.#user_search.focus();
    if (isMobile()) this.#main_nav_toggle.checked = false;
  }

  #makeAreas(area: Area, unprocessed_children: Map<string, CategoryInformation<Badge>[]>) {
    // make the area
    const cat = categoryFromArea(area);
    this.#categories.set(area.name, cat);
    cat.badges.forEach((badge) => {
      badge.data!.ids.forEach((id) => this.#badges.set(id, badge));
    });

    // add any children.
    unprocessed_children.get(area.name)?.forEach((child) => cat.capture(child));

    // sort out the parent situation.
    if (area.parent) {
      const parent = this.#categories.get(area.parent);
      if (parent == undefined) {
        const children = unprocessed_children.get(area.parent) ?? [];
        children.push(cat);
        unprocessed_children.set(area.parent, children);
        return;
      }
      parent.capture(cat);
      return;
    }

    this.#badgesUI.appendChild(cat);
  }

  #makeOther(category: string) {
    if (category == "") return;

    const info = badgeManager.other_category(category);
    const elm = new CategoryInformation<Other>();
    elm.name = category;
    const uiBadges = info.map((badge) => {
      return {
        name: badge.get_name_field.bind(badge),
        name_style: badge.set_name_style.bind(badge),
        information: badge.get_information_field.bind(badge),
        info_style: badge.set_info_style.bind(badge),
        url: badge.link,
        id: badge.id,
        ids: badge.ids,
        wiki: badge.wiki,
        lock_type: badge.lock_type,
        lock_reason: badge.lock_reason,
      } as UIBadgeData<Other>
    });
    elm.addBadges(...uiBadges);
    elm.badges.forEach((badge) => {
      badge.data!.ids.forEach((id) => this.#badges.set(id, badge));
    });
    this.#categories.set(category, elm);
    this.#badgesUI.appendChild(elm);
  }

  /**
   * Load the required data onto the UI in the background.
   */
  load_required_data() {
    const unprocessed_children = new Map<string, CategoryInformation<Tower>[]>();
    // going to bet on the fact that `unprocessed_children` is a reference and not the object itself.
    areaManager.category(Category.Permanent).forEach((area) => this.#makeAreas(area, unprocessed_children));
    areaManager.category(Category.Temporary).forEach((area) => this.#makeAreas(area, unprocessed_children));
    areaManager.category(Category.Other).forEach((area) => this.#makeAreas(area, unprocessed_children));
    badgeManager.other_category().forEach((category) => this.#makeOther(category));

    Array.from(this.#categories.values()).forEach((cat) => {
      // console.log(cat.name);
      if (cat.captured) return;
      cat.changeCategory(0);
    });
  }

}

const ui = new UI();

function categoryFromArea<T extends Badge>(area: Area) {
  const category = new CategoryInformation<T>();
  category.name = area.name;
  category.locked = area.lock_type;
  category.locked_reason = area.lock_reason;
  category.icon = `Assets/Emblems/${area.name.replaceAll(/\s/gm, '')}.webp`;

  const uiBadges = badgeManager.area(area.name).map((badge) => {
    return {
      id: badge.id,
      ids: badge.ids,
      information: badge.get_information_field.bind(badge),
      info_style: badge.set_info_style.bind(badge),
      lock_reason: badge.lock_reason,
      lock_type: badge.lock_type,
      name: badge.get_name_field.bind(badge),
      name_style: badge.set_name_style.bind(badge),
      url: badge.link,
      wiki: badge.wiki
    } as UIBadgeData<T>;
  })

  category.addBadges(...uiBadges);

  return category;
}


export { ui, PreloadState };
