import { Listeners } from "./listeners";

class Logs {
  #tree = {};
  listeners: Listeners<string, (message: string, category?: string, progress?: number) => void>;

  constructor() {
    this.listeners = new Listeners();
  }

  log(message: string, category?: string, progress?: number) {
    console.log(`Received log: ${message} for ${category} at progress ${progress}`);
  }
}

const logs = new Logs();

export { logs };
