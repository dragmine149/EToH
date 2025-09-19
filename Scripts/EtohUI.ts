import { BadgeInformation, CategoryInformation } from "./ui";

enum VisibleState { }

class UI {
  #error_ui: HTMLDivElement;
  #search_ui: HTMLDivElement;
  #badges_ui: HTMLDivElement;

  constructor() {
    this.#error_ui = document.getElementById("errors") as HTMLDivElement;
    this.#search_ui = document.getElementById("search") as HTMLDivElement;
    this.#badges_ui = document.getElementById("badges") as HTMLDivElement;
  }

  show_important_error(message: string) {

  }
}
