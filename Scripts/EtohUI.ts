import { load_required_data } from "./initial";
import { BadgeInformation, CategoryInformation } from "./ui";

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

class UI {
  /**
   * Higher level TS class, built upon the modules of everything else.
   *
   * Designed to keep everything to do with navigation around the UI in one place.
   */

  /** Stuff related to loading the data at initial load. */
  #preload: HTMLDivElement;
  #preload_span: HTMLSpanElement;
  #preload_button: HTMLButtonElement;
  #loaded: boolean;
  set loaded(v) { this.#loaded = v; this.#preload.hidden = v }
  get loaded() { return this.#loaded; }
  #retry_count: 0 | 1 | 2 = 0;

  constructor() {
    this.#preload = document.getElementById("pre-load") as HTMLDivElement;
    this.#preload_span = this.#preload.firstElementChild as HTMLSpanElement;
    this.#preload_button = this.#preload.lastElementChild as HTMLButtonElement;

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
}

const ui = new UI();

export { ui, PreloadState };
