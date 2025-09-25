import { userManager, Tower, Category, Other, numberToType } from "./Etoh";
import { ui, PreloadState } from "./EtohUI";
import { badgeManager, Lock } from "./BadgeManager";
import { ServerAreas, ServerTowers } from "./constants";
import { tryCatch } from "./utils";
import { areaManager, Area } from "./AreaManager";

const regex = /([^,]+),(\d?\d.?\d?\d?),(\[.*\]),(\d)/gm;

function load_area(category: Category, area: ServerAreas) {
  ui.preload(`Loading area ${area.n} of ${category}`, PreloadState.TowerData);
  area.t.forEach((tower) => {
    ui.preload(`Loading tower ${tower} of ${area.n} of ${category}`, PreloadState.TowerData);
    // Custom format parser of `name,diff,[id1,id2,...],type`

    regex.lastIndex = 0;
    let tower_data = regex.exec(tower);
    if (tower_data == null || tower_data.length < 5) {
      console.log(tower_data, tower);
      ui.preload(`Tower data: \`${tower}\` doesn't contain enough info. Skipping`, PreloadState.TowerWarning);
      return;
    }

    let tower_badge = new Tower(
      tower_data[1],
      JSON.parse(tower_data[2]),
      Lock.Unlocked,
      Number.parseInt(tower_data[1]),
      area.n,
      numberToType(Number.parseInt(tower_data[3])),
      category
    );

    badgeManager.addBadge(tower_badge);
  });

  let requirements = {
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

  let area_data = new Area(area.n, area.s == "Windswept Peak" ? "" : area.s, requirements, category);
  areaManager.addArea(area_data);
  ui.preload(`Finish loading area ${area.n} of ${category}`, PreloadState.TowerData);
}

async function loadTowersFromServer() {
  ui.preload("Load towers from server", PreloadState.TowerData);
  let server_tower = await fetch('https://raw.githubusercontent.com/dragmine149/EToH/refs/heads/Data/tower_data.json');

  if (!server_tower.ok) {
    ui.preload(`Failed to fetch due to ${server_tower.statusText}`, PreloadState.Errored);
    return;
  }

  let data = await tryCatch<ServerTowers>(server_tower.json());

  if (data.error) {
    ui.preload(`Failed to parse tower data: ${data.error.message}`, PreloadState.Errored);
    return;
  }

  data.data.areas.permanent.forEach((area) => load_area(Category.Permanent, area));
  data.data.areas.temporary.forEach((area) => load_area(Category.Temporary, area));
  data.data.areas.other.forEach((area) => load_area(Category.Other, area));
}

async function loadOthersFromServer() {
  ui.preload(`Attempting to load other data`, PreloadState.OtherData);
  let server_other = await fetch('data/other_data.json');
  if (!server_other.ok) {
    ui.preload(`Failed to get other data:${server_other.statusText}`, PreloadState.Errored);
    return;
  }

  let data = await tryCatch<ServerOtherParent>(server_other.json());

  if (data.error) {
    ui.preload(`Failed to parse other data: ${data.error.message}`, PreloadState.Errored);
    return;
  }

  data.data.data.forEach((badge) => {
    badgeManager.addBadge(new Other(badge.name, badge.badges, Lock.Unlocked, badge.category));
  })
}


document.addEventListener('DOMContentLoaded', () => {
});

// custom event just to make it rely on something else.
// Doesn't need to have any connection with `preload` as this can be loaded in the background at any time. And isn't technically required to be able to
// use this project.
addEventListener('user_manager_loaded', () => {
  load_user_from_url("initial");

  // console.log(userManager);
  // console.log(userManager.name());
  ui.datalist_add_user(...userManager.name() as string[]);
});

addEventListener('popstate', load_user_from_url.bind(this, "pop"))

function load_user_from_url(orig: string) {
  const url = new URL(location.toString());
  const user = url.searchParams.get("user");
  console.log(`attempting to load ${user} from ${orig}`);
  if (user) ui.load_user(user, true);
}

async function load_required_data() {
  await loadTowersFromServer();
  await loadOthersFromServer();

  ui.preload(`Completed loading of required assets.`, PreloadState.Finished);
  ui.show_required_data();
}

/**
 * Tests to see if the user is on a mobile device by looking at certain parameters.
 * @returns An estimate to if the user is on a mobile device or not.
 */
export const isMobile = (): boolean => {
  // guard: non-browser environment (SSR)
  if (typeof window === 'undefined' || typeof navigator === 'undefined') {
    return false
  }

  const ua = navigator.userAgent || (window as any).opera || ''
  const uaLower = ua.toLowerCase()

  // common mobile user-agent hints
  const mobileUA =
    /iphone|ipad|ipod|android|blackberry|bb10|opera mini|iemobile|wpdesktop|mobile/i

  // touch capability hint
  const hasTouch =
    'ontouchstart' in window ||
    // modern browsers
    (navigator.maxTouchPoints && navigator.maxTouchPoints > 0) ||
    // legacy
    (navigator as any).msMaxTouchPoints > 0

  // narrow screens often indicate mobile
  const smallScreen = Math.min(window.screen.width, window.screen.height) <= 820

  const result = mobileUA.test(uaLower) || (hasTouch && smallScreen)
  return result
}

load_required_data();

globalThis.import_debug = async () => await import('./debug');

export { load_required_data };
