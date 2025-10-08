import { Area, areaManager } from "./AreaManager";
import { Category, userManager } from "./Etoh";
import { isMobile, load_required_data } from "./initial";
import { BadgeInformation, CategoryInformation } from "./ui";
import { UserManager } from "./user";

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

  // Stuff relatied to the displaying of users.
  #user: HTMLDivElement;
  #user_profile: HTMLAnchorElement;
  #user_img: HTMLImageElement;
  #user_mini_search: HTMLDivElement;
  #user_mini_input: HTMLInputElement;
  #user_search: HTMLInputElement;
  #user_search_button: HTMLButtonElement;
  #user_load_error: HTMLSpanElement;
  #user_search_back: HTMLButtonElement;
  #user_mini_button: HTMLButtonElement;
  #user_mini_viewing: HTMLSpanElement;

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
    this.#user_profile = document.getElementById("user-profile") as HTMLAnchorElement;
    this.#user_img = this.#user_profile.firstElementChild as HTMLImageElement;
    this.#user_search = document.getElementById("search_input") as HTMLInputElement;
    this.#user_search_button = document.getElementById("search_button") as HTMLButtonElement;
    this.#user_search_back = document.getElementById("search_back") as HTMLButtonElement;
    this.#user_load_error = document.getElementById("load_errors") as HTMLSpanElement;
    this.#user_mini_search = document.getElementById("mini-search") as HTMLDivElement;
    this.#user_mini_input = this.#user_mini_search.firstElementChild as HTMLInputElement;
    this.#user_mini_button = this.#user_mini_search.lastElementChild as HTMLButtonElement;
    this.#user_mini_viewing = (document.getElementById("viewing") as HTMLDivElement).firstElementChild as HTMLSpanElement;

    // initial settings, bindings, etc that need to be made for everything to work.
    this.#user_profile.hidden = true;
    this.#user_search.onkeydown = this.#submitUserSearch.bind(this);
    this.#user_search.oninput = this.#syncUserSearch.bind(this);
    this.#user_mini_input.oninput = this.#syncUserSearch.bind(this);
    this.#user_search.value = "";
    this.#user_mini_input.value = "";
    this.#user_search_button.onmousedown = this.#submitUserSearch.bind(this);
    this.#user_mini_button.onmousedown = this.#submitUserSearch.bind(this);
    this.#user_search_button.disabled = this.#user_search.value.length <= 0;
    this.#user_search_back.disabled = true;
    this.#user.onclick = () => this.#miniSearch(true);
    this.#user_mini_input.onclick = () => this.#miniSearch(true);
    this.#user_mini_input.onblur = () => this.#miniSearch(false);
    this.#user_mini_input.onfocus = () => this.#user_mini_input.select();
    this.#user_mini_input.onkeydown = this.#submitUserSearch.bind(this);
  }

  /**
   * Syncs the main search with the mini search and vis-versa.
   * @param ev The event. Used to get the value as we have no idea which to overwrite otherwise.
   */
  #syncUserSearch(ev: InputEvent) {
    let target = ev.target as HTMLInputElement;
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
    let children = this.#user_list.children;
    list.filter((user) => children.namedItem(user) == null).forEach((user) => {
      console.log(`Adding ${user} to the user-list`);
      let option = document.createElement("option");
      option.textContent = user;
      option.value = user;
      option.setAttribute("name", user);
      this.#user_list.appendChild(option);
    })
  }

  /**
   * Loads a user via userManager. Updates the UI accordingly etc.
   *
   * There is no "unload" event as loading a new user will unload the old user.
   * @param user_input The user the user inputted.
   * @param popped Has this user been requested to load from a `popstate` event? (If so, we don't do `pushState`)
   */
  async load_user(user_input: string, popped?: boolean) {
    let user = await userManager.find_user(user_input);
    if (user == undefined) { return; }

    this.#user.textContent = user.ui_name;
    this.#user_profile.href = user.link;
    this.#user_img.src = user.profile;

    this.#user_profile.hidden = false;
    this.#user_search_back.disabled = false;
    this.#user_mini_button.style.right = "3.4rem";

    let url = new URL(location.toString());
    url.searchParams.set("user", user.name);
    if (popped == undefined || popped == false) history.pushState(undefined, "", url);
  }

  /**
   * Loads the main menu, should be a really small function but a separate function just in case.
   */
  async main_user_load() {
    this.#search_main.hidden = false;
    if (isMobile()) this.#main_nav_toggle.checked = false;
  }

  show_required_data() {
    (areaManager.category(Category.Permanent)).forEach((area) => {
      let category = document.createElement("category-information") as CategoryInformation<Area>;
      category
    })
  }

}

const ui = new UI();

export { ui, PreloadState };
