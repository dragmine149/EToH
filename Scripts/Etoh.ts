import { etohDB } from ".";
import { Badge } from "./BadgeManager";
import { logs } from "./logs";
import { CLOUD_URL, network, RawBadge } from "./network";
import { User, UserManager } from "./user";

type BadgeId = number;
type CompletedDate = number;

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

class Tower extends Badge {
  #difficulty: number;
  get difficulty() { return this.#difficulty; }
  #area: string;
  get area() { return this.#area; }
  #type: TowerType;
  get type() { return this.#type; }
  #category: Category;
  get category() { return this.category; }

  get shortName() { return shortTowerName(this.name); }

  // get_name_field(hover: boolean) {
  //   return hover ? this.name : this.shortName;
  // }
  // get_information_field(hover: boolean) {
  //   return hover ? `${getDifficulty(this.difficulty)} (${this.difficulty})` : getDifficultyWord(this.difficulty);
  // }


  constructor(name: string, ids: number[], difficulty: number, area: string, type: TowerType, category: Category) {
    super(name, ids);
    this.#difficulty = difficulty;
    this.#area = area;
    this.#type = type;
    this.#category = category;
  }
}



const userManager = new UserManager(etohDB.users, EToHUser);

export { userManager, shortTowerName };
