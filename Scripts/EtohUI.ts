import { userManager } from "./Etoh";
import { load_required_data } from "./initial";
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
  #search_main: HTMLDivElement;
  #user_list: HTMLDataListElement;

  // Stuff relatied to the displaying of users.
  #user: HTMLDivElement;
  #user_profile: HTMLAnchorElement;
  #user_img: HTMLImageElement;
  #user_search: HTMLInputElement;
  #user_search_button: HTMLButtonElement;
  #user_load_error: HTMLSpanElement;
  #user_search_back: HTMLButtonElement;

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

    this.#search_main = document.getElementById("search-main") as HTMLDivElement;
    this.#user_list = document.getElementById("user_list") as HTMLDataListElement;

    this.#user = document.getElementsByTagName("user").item(0) as HTMLDivElement;
    this.#user_profile = document.getElementById("user-profile") as HTMLAnchorElement;
    this.#user_img = this.#user_profile.firstElementChild as HTMLImageElement;
    this.#user_search = document.getElementById("search_input") as HTMLInputElement;
    this.#user_search_button = document.getElementById("search_button") as HTMLButtonElement;
    this.#user_search_back = document.getElementById("search_back") as HTMLButtonElement;
    this.#user_load_error = document.getElementById("load_errors") as HTMLSpanElement;

    this.#user_search.onsubmit = () => this.load_user(this.#user_search.value);
    this.#user_search.onkeydown = (ev) => {
      if (ev.key === 'Enter') this.load_user(this.#user_search.value);
    }

    this.#user_search.oninput = () => {
      // console.log(this.#user_search.value.length);
      this.#user_search_button.disabled = this.#user_search.value.length <= 0;
    }
    this.#user_search_button.onmousedown = () => this.load_user(this.#user_search.value);
    this.#user_search_button.disabled = this.#user_search.value.length <= 0;
    this.#user_search_back.disabled = true;
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

  update_local_user_list(list: string[]) {
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
   */
  async load_user(user_input: string) {
    let user = await userManager.find_user(user_input);
    if (user == undefined) { return; }

    this.#user.textContent = user.ui_name;
    this.#user_profile.href = user.link;
    this.#user_img.src = user.profile;

    this.#user_search_back.disabled = false;
  }

}

const ui = new UI();

export { ui, PreloadState };
