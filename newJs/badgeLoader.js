/**
* @typedef {{[x: string]: (badgeIds|Categories)}} Categories
*/

/**
* Loop through all values in the json and assign them to their categories / badge locations.
* @param {Category} category The category the badge belongs to.
* @param {Categories} towers The tower information index.
*/
function CategoryLoop(category, towers) {
  console.log(category, towers);
  Object.entries(towers).forEach(([key, value]) => {
    if (key.startsWith("$")) return;
    if (key == 'area_information') {
      category.addInformation('area', value);
      return;
    }
    if (value.badge_id != undefined && typeof value.badge_id === 'number') {
      let badge = new Badge(key, value.badge_id);
      badge.addIds(value.old_id);
      category.addBadge(badge);
      return;
    }

    let cat = new Category(key);
    category.addSubCategory(cat);
    CategoryLoop(cat, value);
  })
}


async function loadOtherFromServer() {
  let server_other = await fetch('data/other_data.json');
  if (!server_other.ok) {
    ui.showError(`Failed to fetch tower_data.json: ${server_other.status} ${server_other.statusText}.`, true);
    return;
  }

  /** @type {{data: Categories | null, error: Error | null}} */
  let data = await tryCatch(server_other.json());

  if (data.error) {
    ui.showError(`Failed to parse other_data.json: ${data.error}`, true);
    return;
  }

  let root = new Category('root')
  CategoryLoop(root, data.data)
  return root;
}

async function loadTowersFromServer() {
  let server_tower = await fetch('data/tower_data.json');
  if (!server_tower.ok) {
    ui.showError(`Failed to fetch tower_data.json: ${server_tower.status} ${server_tower.statusText}.`, true);
    return;
  }

  /** @type {{data: Towers | null, error: Error | null}} */
  let data = await tryCatch(server_tower.json());

  if (data.error) {
    ui.showError(`Failed to parse other_data.json: ${data.error}`, true);
    return;
  }

  let root = new Category('root')
  CategoryLoop(root, data.data)
  return root;
}
