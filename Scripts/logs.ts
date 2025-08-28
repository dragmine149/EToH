import { Listeners } from "./listeners";
import { Verbose } from "./verbose.mjs";

class Logs {
  #tree = {};
  listeners: Listeners<string, (message: string, category?: string, progress?: number) => void>;

  constructor() {
    this.listeners = new Listeners();
  }

  log(message: string, category?: string, progress?: number) {

  }
}

const logs = new Logs();

export { logs };
