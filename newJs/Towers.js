/*global Tower, Verbose, badgeManager */
/*eslint no-undef: "error" */
/*exported TowerManager */

class TowerManager {
  /**
  * Returns the word that describes the number.
  * @param {number} difficulty The difficuty of the tower.
  * @returns The word to describe it.
  */
  getDifficultyWord(difficulty) {
    return this.difficulties[Math.trunc(difficulty) - 1];
  }

  /**
  * Translates the number form into a more readable word form. Defaults to "Baseline Unknown" if it can't find anything.
  * @param {number} difficulty The difficulty of the tower
  * @returns {string} The word form of the difficulty
  */
  getDifficulty(difficulty) {
    let stage = Math.trunc(difficulty);
    let sub = difficulty % 1;

    let stageWord = this.difficulties[stage - 1] || "Unknown";
    let subWord = this.subLevels.find(level => sub >= level.threshold)?.name || "Baseline";

    return `${subWord} ${stageWord}`;
  }

  __createUI() {
    let areas = badgeManager.area();
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

        /** @type {HTMLDivElement} */
        let towerClone = clone.querySelector("[tag='badges'] [tag='template']").cloneNode(true);
        towerClone.hidden = false;
        let towerName = towerClone.querySelector("[tag='name']")
        towerName.innerText = tower.shortName;

        let towerDifficulty = towerClone.querySelector("[tag='difficulty']");
        towerDifficulty.innerText = this.getDifficultyWord(tower.difficulty);

        // implement hovering features.
        towerClone.onmouseover = () => {
          towerName.innerText = tower.name;
          towerDifficulty.innerText = `${this.getDifficulty(tower.difficulty)} (${tower.difficulty})`;
        }
        towerClone.onmouseleave = () => {
          towerName.innerText = tower.shortName;
          towerDifficulty.innerText = this.getDifficultyWord(tower.difficulty);;
        }

        tower.ui = towerClone;
        clone.querySelector("[tag='badges']").appendChild(towerClone);
      });

      document.getElementById("towers").appendChild(clone);
    });
  }
}
