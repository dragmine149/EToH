(async () => {
  console.log("Setting up other modules from main website");

  let verbose = await import('https://dragmine149.github.io/Scripts/verbose.mjs');
  console.log(verbose);
  globalThis.Verbose = verbose.Verbose;
  globalThis.logs = verbose.logs;

  let storage = await import('https://dragmine149.github.io/external/storage.js');
  globalThis.DragStorage = storage.DragStorage;
})();
