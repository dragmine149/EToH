import { userManager } from "./Etoh";

async function loadTowersFromServer() {
  let server_tower = await fetch('https://raw.githubusercontent.com/dragmine149/EToH/refs/heads/Data/tower_data.json');
  // let server_tower = await fetch('http://localhost:8081/tower_data.json');
  if (!server_tower.ok) {
    ui.showError(`Failed to fetch tower_data.json: ${server_tower.status} ${server_tower.statusText}.`, true);
    return;
  }

  /** @type {{data: ServerTowers | null, error: Error | null}} */
  let data = await tryCatch(server_tower.json());

  if (data.error) {
    ui.showError(`Failed to parse other_data.json: ${data.error} `, true);
    return;
  }
  Object.entries(data.data.areas).forEach(
    /**
    * @param {[String, ServerAreas[]]} areas
    */
    (areas) => {
      console.log(areas);
      areas[1].forEach((area) => {
        area.t.forEach(/** @param {string} tower */(tower) => {
          let tower_split = tower.split(',');
          let type = numberToType(Number.parseInt(tower_split.pop()));
          /** @type {string[]} */
          let badges = [];
          while (badges.length == 0 || !badges[badges.length - 1].startsWith("[")) {
            badges.push(tower_split.pop());
          }
          // console.log(badges);
          badges = JSON.parse(badges.reverse().join(','));
          let diff = Number.parseFloat(tower_split.pop());
          let name = tower_split.join(',');
          name = addTowerType(name, type);


          // let type = numberToType(Number.parseInt(tower_split[3]));
          // let name = addTowerType(tower_split[0], type);

          // console.warn(areas[0]);
          let tower_badge = new Tower(name, badges, diff, area.n, type, areas[0]);
          // console.log(tower_badge);
          badgeManager.addBadge(tower_badge);
        });
        area.requirements = {
          difficulties: {},
          points: 0,
        }
        area.requirements.points = area.r.p;
        area.requirements.difficulties.easy = area.r.ds.e ? area.r.ds.e : 0;
        area.requirements.difficulties.medium = area.r.ds.m ? area.r.ds.m : 0;
        area.requirements.difficulties.hard = area.r.ds.h ? area.r.ds.h : 0;
        area.requirements.difficulties.difficult = area.r.ds.d ? area.r.ds.d : 0;
        area.requirements.difficulties.challenging = area.r.ds.c ? area.r.ds.c : 0;
        area.requirements.difficulties.intense = area.r.ds.i ? area.r.ds.i : 0;
        area.requirements.difficulties.remorseless = area.r.ds.r ? area.r.ds.r : 0;
        area.requirements.difficulties.insane = area.r.ds.s ? area.r.ds.s : 0;
        area.requirements.difficulties.extreme = area.r.ds.x ? area.r.ds.x : 0;
        area.requirements.difficulties.terrifying = area.r.ds.t ? area.r.ds.t : 0;
        area.requirements.difficulties.catastrophic = area.r.ds.a ? area.r.ds.a : 0;
        area.s = area.s == "Windswept Peak" ? "" : area.s;
        console.log(area.requirements);

        areaManager.addArea(new Area(area.n, area.s, area.requirements));
      })
    })
}

async function loadOthersFromServer() {
  let server_other = await fetch('data/other_data.json');
  if (!server_other.ok) {
    ui.showError(`Failed to fetch tower_data.json: ${server_other.status} ${server_other.statusText}.`, true);
    return;
  }

  /** @type {{data: ServerOtherParent | null, error: Error | null}} */
  let data = await tryCatch(server_other.json());

  if (data.error) {
    ui.showError(`Failed to parse other_data.json: ${data.error} `, true);
    return;
  }

  data.data.data.forEach((badge) => {
    badgeManager.addBadge(new Other(badge.name, badge.badges, badge.category));
  })
}


document.addEventListener('DOMContentLoaded', () => {
  const url = new URL(location.toString());
  const user = url.searchParams.get("user");
  if (user) userManager.find_user(Number.isNaN(user) ? user : Number(user));
})
