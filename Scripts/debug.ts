import { GenericManager } from "./GenericManager";
import { Badge, badgeManager } from "./BadgeManager";
import { Area, areaManager } from "./AreaManager";
import { ui } from "./EtohUI";

console.log("Setting up debug!");

globalThis.debug = {
  GenericManager,
  badge: {
    Badge, badgeManager
  },
  area: {
    Area, areaManager
  },
  ui
}
