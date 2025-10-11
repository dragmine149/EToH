import { etohDB } from ".";
import { Badge, BadgeManager, Lock } from "./BadgeManager";
import { DIFFICULTIES, SUB_LEVELS } from "./constants";
import { logs } from "./logs";
import { CLOUD_URL, network, RawBadge } from "./network";
import { User, UserManager } from "./user";

type BadgeId = number;
type CompletedDate = number;

/**
 * An extension of type User. Allows for more control over things that couldn't have been done in base user class.

 * Note: This is not saved to storage directly, instead it is kinda saved in 2 parts in a way.
 */
class EToHUser extends User {
  completed: Map<BadgeId, CompletedDate>;

  /**
   * Load badges from the server, only those not completed.
   * @param badges Badges not yet completed.
   * @param callback Callback once we get a badge.
   */
  async loadServerBadges(badges: number[], callback: (badge: RawBadge) => void) {
    logs.log(`Attempting to load badges from server for ${this.name}`, `user`, 0);
    await network.requestStream(new Request(`${CLOUD_URL}/badges/${this.id}/all`, {
      method: 'POST',
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        'badgeids': badges
      })
    }), async (line) => {
      const badgeInfo = JSON.parse(line) as RawBadge;
      if (this.completed.has(badgeInfo.badgeId)) {
        logs.log(`Already have badge stored: ${badgeInfo.badgeId}`, `user`, 50);
        return;
      }

      this.completed.set(badgeInfo.badgeId, badgeInfo.date);

      callback(badgeInfo);
    });
    logs.log(`Storing data in the local database...`, `user`, 90);
    await this.saveDatabase();
    logs.log(`Completed load from network!`, `user`, 100);
  }

  /**
   * Load badges from the local database.
   * This is intended for quicker loading whilst we wait on server.
   */
  async loadDatabaseBadges() {
    const database = await etohDB.badges.where({ userId: this.id }).toArray();
    database.forEach((b) => {
      this.completed.set(b.badgeId, b.date);
    });
    // Note, no need to save the database we just loaded.
  }

  /**
   * Save the database for offline loading.
   */
  async saveDatabase() {
    // got to map it to a format the database works better with.
    const badgeArray = Array.from(this.completed.entries()).map(([badgeId, date]) => ({
      badgeId,
      date,
      userId: this.id
    }));

    await etohDB.badges.bulkPut(badgeArray)
      .then((v) => { console.info(`Finished storing all badges in database`); console.log(v) })
      .catch(e => {
        console.error(`Failed to add in ${badgeArray.length - e.failures.length} badges`, e);
      });
  }
}

enum TowerType {
  MiniTower, Steeple, Tower, Citadel, Obelisk, Other
}
enum Category {
  Permanent, Temporary, Other
}

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
* Returns the word that describes the number.
* @param difficulty The difficuty of the tower.
* @returns The word to describe it.
*/
function getDifficultyWord(difficulty: number) { return DIFFICULTIES[Math.trunc(difficulty) - 1] ?? "Unknown_Difficulty"; }

/**
* Translates the number form into a more readable word form. Defaults to "Baseline Unknown" if it can't find anything.
* @param difficulty The difficulty of the tower
* @returns The word form of the difficulty
*/
function getDifficulty(difficulty: number) {
  const stage = Math.trunc(difficulty);
  const sub = difficulty % 1;

  const stageWord = DIFFICULTIES[stage - 1] || "Unknown_Difficulty";
  const subWord = SUB_LEVELS.find(level => sub >= level.threshold)?.name || "Baseline";

  return `${subWord} ${stageWord}`;
}

/**
* Returns what type the tower is from its name. (Please don't make this too confusing EToH Devs...)
* @param name The name of the tower.
* @param type The type to use when we can't determine from name or from badge.
* @returns The type of tower.
*/
function getTowerType(name: string, type: string) {
  if (name.startsWith("Steeple")) return TowerType.Steeple;
  if (name.startsWith('Tower of') || name == 'Thanos Tower') return TowerType.Tower;
  if (name.startsWith('Citadel of')) return TowerType.Citadel;
  if (name.startsWith('Obeisk of')) return TowerType.Obelisk;
  const badge = badgeManager.name(name)[0];
  if (badge instanceof Tower) return badge.type;
  return type;
}

/**
 * Adds back the `xxx of` stripped in the rust program to save space.
 * @param name The name of the tower.
 * @param type The type of the tower.
 * @returns The full name.
 */
function addTowerType(name: string, type: TowerType) {
  switch (type) {
    case TowerType.Citadel: return `Citadel of ${name}`;
    case TowerType.Tower: return `Tower of ${name}`;
    case TowerType.Steeple: return `Steeple of ${name}`;
    case TowerType.Obelisk: return `Obelisk of ${name}`;
  }
  return name;
}
/**
* Returns the amount of points that the type provided is worth.
* @param {TowerType} type The type of tower.
*/
function getTowerPoints(type: TowerType) {
  switch (type) {
    case TowerType.Obelisk: return 3;
    case TowerType.Citadel: return 2;
    case TowerType.Tower: return 1;
    case TowerType.Steeple: return 0.5;

    default:
    case TowerType.MiniTower:
    case TowerType.Other: return 0;
  }
}

/**
 * Converts a number from the server to use as a more readable type.
 * @param num The server provided type of this tower.
 * @returns The local conversion of this tower.
 */
function numberToType(num: number) {
  switch (num) {
    case 0: return TowerType.MiniTower;
    case 1: return TowerType.Steeple;
    case 2: return TowerType.Tower;
    case 3: return TowerType.Citadel;
    case 4: return TowerType.Obelisk;
    default: return TowerType.Other;
  }
}

/**
 * An extension of Badge containing specific information for the tower.
 */
class Tower extends Badge {
  #difficulty: number;
  get difficulty() { return this.#difficulty; }
  #area: string;
  get area() { return this.#area; }
  #type: TowerType;
  get type() { return this.#type; }
  #category: Category;
  get category() { return this.#category; }

  get shortName() { return shortTowerName(this.name); }

  get_name_field(hover: boolean) {
    return hover ? this.name : this.shortName;
  }
  get_information_field(hover: boolean) {
    const word = getDifficultyWord(this.difficulty);
    return hover ? `${getDifficulty(this.difficulty)} (${this.difficulty})` : word;
  }

  set_info_style(): string {
    const lword = getDifficultyWord(this.difficulty).toLowerCase();
    return `background: var(--difficulty-${lword}); color: var(--difficulty-${lword}-text);`;
  }


  constructor(name: string, ids: number[], lock_type: Lock, difficulty: number, area: string, type: TowerType, category: Category, lock_reason?: string, wiki?: URL) {
    super(name, ids, lock_type, wiki, lock_reason);
    this.#difficulty = difficulty;
    this.#area = area;
    this.#type = type;
    this.#category = category;
  }
}

class Other extends Badge {
  #category: string;
  get category() { return this.#category; }

  constructor(name: string, ids: number[], lock_type: Lock, category: string, lock_reason?: string) {
    super(name, ids, lock_type, undefined, lock_reason);
    this.#category = category;
  }
}

class EtohBadgeManager extends BadgeManager<Tower | Other, Category> {
  category!: { (): Category[], (item?: Category): (Tower | Other)[] };
  area!: { (): string[], (item: string): Tower[] };

  constructor() {
    super();

    this.addFilter("category", badge => badge.category);
    this.addFilter("area", badge => {
      if (badge instanceof Tower) {
        return badge.area;
      }
      return "";
    });
  }
}

// defined here to allow for access to said database and userclass.
const userManager = new UserManager(etohDB.users, EToHUser);
const badgeManager = new EtohBadgeManager();

export { userManager, badgeManager, shortTowerName, Tower, Category, Other, numberToType, addTowerType };
