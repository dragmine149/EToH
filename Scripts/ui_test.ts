import { CategoryInformation, UIBadgeData, CategoryData, Count, BadgeInformation } from "./ui";
import { Lock, Badge } from "./BadgeManager";

/**
* Returns the shortened version of the text, in accordance to tower format.
* @param text The text to shorten.
*/
function shortTowerName(tower_name: string) {
  // Tower codes are made up of:
  // each word
  return tower_name.split(/[\s-]/gm)
    // lowered
    .map(word => word.toLowerCase())
    // for 'of' and 'and' to be lower, and the rest upper.
    .map(word => (word == 'of' || word == 'and') ? word[0] : word[0].toUpperCase())
    // and combined.
    .join('');
}


/**
 * A function which generates random category data.
 */
function random_Category(): CategoryData {
  const names = ["Forest Path", "Desert Storm", "Mountain Peak", "Ocean Waves", "City Center"];
  const randomName = names[Math.floor(Math.random() * names.length)];
  const locks: Lock[] = [Lock.Unlocked, Lock.Temporary, Lock.Another];
  const randomLock = locks[Math.floor(Math.random() * locks.length)];

  return {
    name: randomName,
    lock_type: randomLock,
  };
}

/**
 * A function which generates random badge data.
 */
function random_badges(): UIBadgeData<Badge>[] {
  const wordList = ["Forest", "Desert", "Mountain", "Ocean", "City", "Ancient", "Lost", "Forgotten", "Shadow", "Crystal", "Iron", "Steel", "Stone", "Fire", "Ice", "Wind", "Water", "Earth", "Sky", "Void", "and"];
  const badgeCount = Math.floor(Math.random() * 10) + 3; // Random number of badges between 1 and 5
  const locks: Lock[] = [Lock.Unlocked, Lock.Temporary, Lock.Another];

  return Array.from({ length: badgeCount }, () => {
    const wordCount = Math.floor(Math.random() * 4) + 1; // Random number of words between 1 and 4
    const towerNameWords = Array.from({ length: wordCount }, () => wordList[Math.floor(Math.random() * wordList.length)]);
    const towerName = towerNameWords.join(" ");
    const id = Math.floor(Math.random() * 1000);
    const completed = Math.random() < 0.7 ? Date.now() - Math.floor(Math.random() * 365 * 24 * 60 * 60 * 1000) : 0;
    const lock_type = locks[Math.floor(Math.random() * locks.length)];

    return {
      name: (hover: boolean) => hover ? `Tower of ${towerName}` : shortTowerName(`Tower of ${towerName}`),
      information: (hover: boolean) => `Information about Tower of` + (hover ? ` ${towerName} (Hovered)` : ` ${shortTowerName(towerName)}`),
      url: `https://example.com/${towerName.toLowerCase().replace(" ", "_")}`,
      id: id,
      completed: completed,
      lock_type: lock_type,
      lock_reason: `Locked because reasons.`,
    };
  });
}

const elms: CategoryInformation<Badge>[] = [];

const createCI = (recursive?: boolean) => {
  console.log('creating new element');
  const ci = document.createElement('category-info') as CategoryInformation<Badge>;
  const data = random_Category();
  ci.data = data;
  ci.count = Math.random() >
    0.33 ? Count.None : (Math.random() > 0.66 ? Count.Numbers : Count.Percent);

  console.log(data.name);
  console.log(recursive);
  ci.addBadges(...random_badges());
  if (recursive === true) {
    console.group("cat sub gen");
    const group = createCI(false);
    console.groupEnd();
    console.log(group);
    ci.addCategory(group);
  }
  if (recursive === undefined || recursive === true) {
    document.body.appendChild(ci);
    elms.push(ci);
  }
  return ci;
}

document.addEventListener('DOMContentLoaded', () => {
  document.getElementById("e")?.addEventListener('click', createCI.bind(this, undefined));
  for (let i = 0; i < 2; i++) {
    createCI(true);
  }
});
