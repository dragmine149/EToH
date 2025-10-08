import { GenericManager } from "./GenericManager";
import { Badge } from "./BadgeManager";
import { Area, areaManager } from "./AreaManager";
import { User } from "./user";
import { userManager, badgeManager, Tower, Other } from "./Etoh";
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
  ui,
  user: {
    userManager,
    User
  },
  Tower, Other
}
