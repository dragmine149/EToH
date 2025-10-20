import { GenericManager } from "./Core/GenericManager";
import { Badge } from "./ETOHBridge/BadgeManager";
import { Area, areaManager } from "./ETOHBridge/AreaManager";
import { User } from "./Core/user";
import { userManager, badgeManager, Tower, Other } from "./ETOH/Etoh";
import { ui } from "./ETOH/EtohUI";

console.log("Setting up debug!");

globalThis.debug = {
  GenericManager,
  badge: {
    Badge, badgeManager
  },
  area: {
    Area, areaManager
  },
  ui,
  user: {
    userManager,
    User
  },
  Tower, Other
}
