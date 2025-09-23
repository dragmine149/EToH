/**
 * An extension of the browser Console to allow displaying messages in the UI as well.
 *
 * NOTE: Not all functions modify the UI, Some are here just as a passthrough.
 * NOTE: Most functions have been commented out as they are unlikely to be used. The important ones have been left in.
 */
class Console {
  set status(v) { this.#status = v; }
  get status() {
    if (this.#status == undefined) {
      this.#status = document.getElementById("status") as HTMLDivElement | undefined
    }
    return this.#status;
  }
  #status?: HTMLDivElement;

  /**
   * The `console.assert()` static method writes an error message to the console if the assertion is false. If the assertion is true, nothing happens.
   * @param condition Any boolean expression. If the assertion is false, a generic message indicating assertion failure is written to the console.
   * @param data The message / object / variable to output.
   */
  // assert(condition: boolean, ...data: any[]) {
  //   console.assert(condition, ...data);
  //   if (this.status) this.status.textContent = JSON.stringify(data);
  // }
  /**
   * The `console.clear()` static method clears the console if possible.
   */
  // clear() {
  //   console.clear();
  //   if (this.status) this.status.textContent = "";
  // }
  /**
   * **PASSTHROUGH**
   *
   * The `console.count()` static method logs the number of times that this particular call to `count()` has been called.
   * @param label A string. If supplied, `count()` outputs the number of times it has been called with that label. If omitted, `count()` behaves as though it was called with the "default" label.
   */
  // count(label?: string) {
  //   console.count(label);
  // }
  /**
   * **PASSTHROUGH**
   *
   * The `console.countReset()` static method resets counter used with `console.count()`.
   * @param label A string. If supplied, `countReset()` resets the count for that label to `0`. If omitted, countReset() resets the default counter to `0`.
   */
  // countReset(label?: string) {
  //   console.countReset(label);
  // }
  /**
   * The `console.debug()` static method outputs a message to the console at the "debug" log level. The message is only displayed to the user if the console is configured to display debug output. In most cases, the log level is configured within the console UI. This log level might correspond to the Debug or Verbose log level.
   * @param data The message / object / variable to output.
   */
  debug(...data: any[]) {
    console.debug(...data);
    if (this.status) this.status.textContent = JSON.stringify(data);
  }
  /**
   * **PASSTHROUGH**
   *
   * The `console.dir()` static method displays a list of the properties of the specified JavaScript object. In browser consoles, the output is presented as a hierarchical listing with disclosure triangles that let you see the contents of child objects.
   * @param data A JavaScript object whose properties should be printed.
   */
  // dir(item: any, options?: any) {
  //   console.dir(item, options);
  // }
  /**
   * **PASSTHROUGH**
   *
   * The `console.dirxml()` static method displays an interactive tree of the descendant elements of the specified XML/HTML element. If it is not possible to display as an element the JavaScript Object view is shown instead. The output is presented as a hierarchical listing of expandable nodes that let you see the contents of child nodes.
   * @param data A JavaScript object whose properties should be output.
   */
  // dirxml(...data: any[]) {
  //   console.dirxml(...data);
  // }
  error(...data: any[]) {
    console.error(...data);
    if (this.status) this.status.textContent = JSON.stringify(data);
  }
  /**
   * The `console.group()` static method creates a new inline group in the Web console log, causing any subsequent console messages to be indented by an additional level, until `console.groupEnd()` is called.
   * @param label Label for the group.
   */
  // group(label?: string) {
  //   console.group(label);
  // }
  /**
   * The `console.groupCollapsed()` static method creates a new inline group in the console. Unlike `console.group()`, however, the new group is created collapsed. The user will need to use the disclosure button next to it to expand it, revealing the entries created in the group.
   * @param label Label for the group.
   */
  // groupCollapsed(label?: string) {
  //   console.groupCollapsed(label);
  // }
  /**
   * The `console.groupEnd()` static method exits the current inline group in the console. See Using groups in the console in the console documentation for details and examples.
   */
  // groupEnd() {
  //   console.groupEnd();
  // }
  /**
   * The `console.info()` static method outputs a message to the console at the "info" log level. The message is only displayed to the user if the console is configured to display info output. In most cases, the log level is configured within the console UI. The message may receive special formatting, such as a small "i" icon next to it.
   * @param data The message / object / variable to output.
   */
  info(...data: any[]) {
    console.info(...data);
    if (this.status) this.status.textContent = JSON.stringify(data);
  }
  /**
   * The `console.log()` static method outputs a message to the console.
   * @param data The message / object / variable to output.
   */
  log(...data: any[]) {
    console.log(...data);
    if (this.status) this.status.textContent = JSON.stringify(data);
  }
  /**
   * **PASSTHROUGH**
   *
   * The `console.table()` static method displays tabular data as a table.
   * @param data The data to display. This must be either an array or an object. Each item in the array, or property in the object, is represented by a row in the table. The first column in the table is labeled (index) and its values are the array indices or the property names.
   */
  table(...data: any[]) {
    console.table(...data);
  }
  /**
   * **PASSTHROUGH**
   *
   * The `console.time()` static method starts a timer you can use to track how long an operation takes. You give each timer a unique name, and may have up to 10,000 timers running on a given page. When you call `console.timeEnd()` with the same name, the browser will output the time, in milliseconds, that elapsed since the timer was started.
   * @param label A string representing the name to give the new timer. This will identify the timer; use the same name when calling `console.timeEnd()` to stop the timer and get the time output to the console. If omitted, the label "default" is used.
   */
  // time(label?: string) {
  //   console.time(label);
  // }
  /**
   * **PASSTHROUGH**
   *
   * The `console.timeEnd()` static method stops a timer that was previously started by calling `console.time()`.
   * @param label A string representing the name of the timer to stop. Once stopped, the elapsed time is automatically displayed in the console along with an indicator that the time has ended. If omitted, the label "default" is used.
   */
  // timeEnd(label?: string) {
  //   console.timeEnd(label);
  // }
  /**
   * **PASSTHROUGH**
   *
   * The `console.timeLog()` static method logs the current value of a timer that was previously started by calling `console.time()`.
   * @param label The name of the timer to log to the console. If this is omitted the label "default" is used.
   * @param data Additional values to be logged to the console after the timer output.
   */
  // timeLog(label?: string, ...data: any[]) {
  //   console.timeLog(label, ...data);
  // }
  trace(...data: any[]) {
    console.trace(...data);
    if (this.status) this.status.textContent = JSON.stringify(data);
  }
  warn(...data: any[]) {
    console.warn(...data);
    if (this.status) this.status.textContent = JSON.stringify(data);
  }
}

let console2 = new Console();
export { console2 as console };
