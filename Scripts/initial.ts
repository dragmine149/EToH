import { userManager, Tower, Category, Other, numberToType, badgeManager, addTowerType } from "./Etoh";
import { ui, PreloadState } from "./EtohUI";
import { Lock } from "./BadgeManager";
import { ServerAreas, ServerOtherParent, ServerTowers } from "./constants";
import { tryCatch } from "./utils";
import { areaManager, Area } from "./AreaManager";

/**Custom regex is formatted as such to handle `"{name},{difficult},[{badges},{more badges}],{type}"`
 * @see https://regex101.com/r/Y1GLyf/1 for examples
 */
const regex = /([^,]+),(\d?\d.?\d?\d?),(\[.*\]),(\d)/gm;

/**
 * Helper function to load areas from the server as there is a lot to do.
 * @param category The main type this area comes under.
 * @param area Information about the area from the server.
 */
function load_area(category: Category, area: ServerAreas) {
  ui.preload(`Loading area ${area.n} of ${category}`, PreloadState.TowerData);
  area.t.forEach((tower) => {
    ui.preload(`Loading tower ${tower} of ${area.n} of ${category}`, PreloadState.TowerData);

    regex.lastIndex = 0; // due to global regex.
    const tower_data = regex.exec(tower);
    if (tower_data == null || tower_data.length < 5) {
      // we should be worried if this happens, but to the user we don't want to worry them too much.
      console.warn(`Failed to load tower data as the regex didn't full parse it. Please try again tomorrow and/or report an issue on github. Data in question: `, tower_data, tower);
      ui.preload(`Tower data: \`${tower}\` doesn't contain enough info. Skipping`, PreloadState.TowerWarning);
      return;
    }
    const tower_type = numberToType(Number.parseInt(tower_data[4]));

    const tower_badge = new Tower(
      addTowerType(tower_data[1], tower_type),
      JSON.parse(tower_data[3]),
      Lock.Unlocked,
      Number.parseInt(tower_data[2]),
      area.n,
      tower_type,
      category
    );
    console.log(tower_data, tower_badge);

    badgeManager.addBadge(tower_badge);
  });

  // this is just converting the "database" format into a format easier to work with.
  const requirements = {
    difficulties: {
      easy: area.r.ds.e,
      medium: area.r.ds.m,
      hard: area.r.ds.h,
      difficult: area.r.ds.d,
      challenging: area.r.ds.c,
      intense: area.r.ds.i,
      remorseless: area.r.ds.r,
      insane: area.r.ds.s,
      extreme: area.r.ds.x,
      terrifying: area.r.ds.t,
      catastrophic: area.r.ds.a,
    },
    points: area.r.p,
  }

  const area_data = new Area(area.n, area.s == "Windswept Peak" ? "" : area.s, requirements, category, Lock.Unlocked);
  areaManager.addArea(area_data);
  ui.preload(`Finish loading area ${area.n} of ${category}`, PreloadState.TowerData);
}

/**
 * Sends a network request to get all the towers and starts loading them into memory for immedite and future use.
 */
async function loadTowersFromServer() {
  ui.preload("Load towers from server", PreloadState.TowerData);
  const server_tower = await fetch('https://raw.githubusercontent.com/dragmine149/EToH/refs/heads/Data/tower_data.json');

  if (!server_tower.ok) {
    ui.preload(`Failed to fetch due to ${server_tower.statusText}`, PreloadState.Errored);
    return;
  }

  const data = await tryCatch<ServerTowers>(server_tower.json());

  if (data.error) {
    ui.preload(`Failed to parse tower data: ${data.error.message}`, PreloadState.Errored);
    return;
  }

  // This is done in order of priority.
  data.data.areas.permanent.forEach((area) => load_area(Category.Permanent, area));
  data.data.areas.temporary.forEach((area) => load_area(Category.Temporary, area));
  data.data.areas.other.forEach((area) => load_area(Category.Other, area));
}

/**
 * Of cause, towers aren't everything. So load all the other badges and process them slightly differently.
 */
async function loadOthersFromServer() {
  ui.preload(`Attempting to load other data`, PreloadState.OtherData);
  const server_other = await fetch('data/other_data.json');
  if (!server_other.ok) {
    ui.preload(`Failed to get other data:${server_other.statusText}`, PreloadState.Errored);
    return;
  }

  const data = await tryCatch<ServerOtherParent>(server_other.json());

  if (data.error) {
    ui.preload(`Failed to parse other data: ${data.error.message}`, PreloadState.Errored);
    return;
  }

  data.data.data.forEach((badge) => {
    badgeManager.addBadge(new Other(badge.name, badge.badges, Lock.Unlocked, badge.category));
  })
}


// custom event just to make it rely on something else.
// Doesn't need to have any connection with `preload` as this can be loaded in the background at any time. And isn't technically required to be able to
// use this project.
addEventListener('user_manager_loaded', () => {
  load_user_from_url("initial");

  // console.log(userManager);
  // console.log(userManager.name());
  ui.datalist_add_user(...userManager.name());
});

addEventListener('popstate', load_user_from_url.bind(this, "pop"))

/**
 * Helper function to load user based on the URL.
 * @param orig The place this got used from. Used for debugging purposes.
 */
function load_user_from_url(orig: string) {
  const url = new URL(location.toString());
  const user = url.searchParams.get("user");
  console.log(`attempting to load ${user} from ${orig}`);
  if (user) ui.load_user(user, true);
}

/**
 * Global function to load data and keep track of it.
 * Can also be used run again if the data failed to load, hence the export.
 */
async function load_required_data() {
  await loadTowersFromServer();
  await loadOthersFromServer();

  ui.preload(`Completed loading of required assets.`, PreloadState.Finished);
  ui.load_required_data();
}

/**
 * Tests to see if the user is on a mobile device by looking at certain parameters.
 * @returns An estimate to if the user is on a mobile device or not.
 * @author T3 Chat (GPT-5 mini)
 */
export const isMobile = (): boolean => {
  // SSR guard
  if (typeof window === 'undefined' || typeof navigator === 'undefined') {
    return false
  }

  const ua = navigator.userAgent ?? ''
  const uaLower = ua.toLowerCase()

  // basic UA hint for phones/tablets
  const mobileUA = /iphone|ipad|ipod|android|mobile/i

  // modern touch detection and coarse pointer hint
  const hasTouch = (navigator.maxTouchPoints ?? 0) > 0
  const coarsePointer = window.matchMedia?.('(pointer: coarse)').matches ?? false

  const smallScreen = Math.min(window.screen.width, window.screen.height) <= 820

  return mobileUA.test(uaLower) || hasTouch || coarsePointer || smallScreen
}

// some simple, auto run functions.
load_required_data();
// Console only function for debugging purposes. Separated out to reduce overhead + whatever.
globalThis.import_debug = async () => await import('./debug');

export { load_required_data };
