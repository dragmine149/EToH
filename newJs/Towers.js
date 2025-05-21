/*global Tower, Verbose, badgeManager, DIFFICULTIES, SUB_LEVELS, areaManager */
/*eslint no-undef: "error" */
/*exported TowerManager */

class TowerManager {
  /**
  * Returns the word that describes the number.
  * @param {number} difficulty The difficuty of the tower.
  * @returns The word to describe it.
  */
  getDifficultyWord(difficulty) {
    return DIFFICULTIES[Math.trunc(difficulty) - 1];
  }

  /**
  * Translates the number form into a more readable word form. Defaults to "Baseline Unknown" if it can't find anything.
  * @param {number} difficulty The difficulty of the tower
  * @returns {string} The word form of the difficulty
  */
  getDifficulty(difficulty) {
    let stage = Math.trunc(difficulty);
    let sub = difficulty % 1;

    let stageWord = DIFFICULTIES[stage - 1] || "Unknown";
    let subWord = SUB_LEVELS.find(level => sub >= level.threshold)?.name || "Baseline";

    return `${subWord} ${stageWord}`;
  }

  /**
  * Create the UI with all the towers and everything.
  */
  createUI() {
    let parents = areaManager.parent();
    parents.forEach((parent) => {
      let node = areaManager.name(parent)[0];
      if (node.background_ui) return;

      let background = document.createElement("div");
      background.classList.add("parent-background");
      node.background_ui = background;

      // document.getElementById("towers").appendChild(background);
    })

    let areas = areaManager.name();
    areas.forEach(area => {
      let node = areaManager.name(area)[0];
      if (node.ui) return;

      // list of all towers for this area.
      /** @type {Tower[]} */
      let towers = badgeManager.area(area);

      // clone the template ui and update it so its unique.
      /** @type {HTMLDivElement} */
      let clone = document.getElementById("category").cloneNode(true);
      clone.id = `area-${area}`;
      clone.hidden = false;

      /** @type {HTMLSpanElement} */
      let title = clone.querySelector("[tag='title']");
      title.innerText = area;

      towers.filter(tower => !tower.ui).forEach((tower) => {
        // debugger;
        if (tower.difficulty >= 100) return; // AKA: Towers which have not been added yet.

        /** @type {HTMLDivElement} */
        let towerClone = clone.querySelector("[tag='badges'] [tag='tower']").cloneNode(true);
        towerClone.hidden = false;
        let towerName = towerClone.querySelector("[tag='name']")
        towerName.innerText = tower.shortName;

        let towerDifficulty = towerClone.querySelector("[tag='difficulty']");
        towerDifficulty.innerText = this.getDifficultyWord(tower.difficulty);
        towerDifficulty.classList.add("difficulty", this.getDifficultyWord(tower.difficulty));

        // implement hovering features.
        towerClone.onmouseover = () => {
          towerName.innerText = tower.name;
          towerDifficulty.innerText = `${this.getDifficulty(tower.difficulty)} (${tower.difficulty})`;
        }
        towerClone.onmouseleave = () => {
          towerName.innerText = tower.shortName;
          towerDifficulty.innerText = this.getDifficultyWord(tower.difficulty);;
        }

        const mouseOverEvent = new Event('mouseover');
        towerClone.dispatchEvent(mouseOverEvent);
        tower.ui = towerClone;
        clone.querySelector("[tag='badges']").appendChild(towerClone);
      });

      node.ui = clone;
      // console.log(clone.querySelectorAll("[tag='tower']"), clone.querySelectorAll("[tag='tower']").length, node);
      node.valid = clone.querySelectorAll("[tag='tower']").length > 1;
      if (node.valid === false) return;

      // console.log(node);
      // console.log(node.parent ?? area);
      // debugger;
      let parentNode = areaManager.name(node.parent ?? area)[0];
      if (parentNode.background_ui) {
        parentNode.background_ui.appendChild(clone);
        node.ui_parent = node.parent ? true : false;
        return;
      }

      // document.getElementById("towers").appendChild(clone);
    });

    let tallest = 0;
    areas.forEach((area) => {
      let node = areaManager.name(area)[0];
      // console.log(node);
      if (node.valid === false) return;

      // console.log(node);
      if (!node.ui_parent) {
        document.getElementById("towers").appendChild(node.background_ui ?? node.ui);
      }

      node.ui.style.width = node.ui.clientWidth + 'px';
      tallest = Math.max(tallest, node.ui.clientHeight);
      // console.log(node.ui.style.width);
      node.ui.querySelectorAll("[tag='tower']").forEach(tower => {
        const mouseLeaveEvent = new Event('mouseleave');
        tower.dispatchEvent(mouseLeaveEvent);
      });
    });

    areas.forEach((area) => {
      let node = areaManager.name(area)[0];
      node.ui.style.height = tallest + 'px';
    })
  }

  /**
  * Load the UI specifically for that user.
  * @param {EToHUser} user The user to load.
  */
  loadUI(user) {
    this.verbose.info(`Loading user... (${user.ui_name})`);
    document.getElementsByTagName("user")[0].innerText = user.ui_name;

    document.getElementById("search").hidden = true;
    document.getElementById("towers").hidden = false;
    this.createUI();

    let completed = user.completed.map(b => b.badgeId);

    badgeManager.ids().forEach((id) => {
      let tower = badgeManager.ids(id)[0];
      this.verbose.debug(tower.ui);
      if (!tower.ui) return;
      tower.ui.querySelector("[tag='name']").classList.remove("completed");
      if (completed.some(v => tower.ids.includes(v))) {
        tower.ui.querySelector("[tag='name']").classList.add("completed");
      }
    });

    this.verbose.info("Finish loading user!")
  }

  loadBadge(badge_id, state) {
    let badge = badgeManager.ids(badge_id)[0];
    if (!badge.ui) return;
    let ui = badge.ui.querySelector("[tag='name']");
    if (state) {
      ui.classList.add("completed");
      return;
    }
    ui.classList.remove("completed");
  }

  unloadUI() {
    this.verbose.info("Unloading tower ui!");
    document.getElementById("search").hidden = false;
    document.getElementById("towers").hidden = true;
    document.getElementsByTagName("user")[0].innerText = "No-one!";
    this.verbose.info("Finished unloading towers ui!");
  }

  constructor() {
    this.verbose = new Verbose("TowerManager", "#7842dc");
  }
}
