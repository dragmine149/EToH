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

  __createUI() {
    let parents = areaManager.parent();
    parents.forEach((parent) => {
      let background = document.createElement("div");
      background.classList.add("parent-background");
      let node = areaManager.name(parent)[0];
      node.background_ui = background;

      // document.getElementById("towers").appendChild(background);
    })

    let areas = areaManager.name();
    areas.forEach(area => {
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

      towers.forEach((tower) => {
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

      let node = areaManager.name(area)[0];
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

  constructor() {
    this.verbose = new Verbose("TowerManager", "#7842dc");
  }
}
