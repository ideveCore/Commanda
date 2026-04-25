export class KeyboardShortcutManager {
  constructor() {}

  /**
   *
   * @param {string} key
   * @param {function(): void} callback
   */
  addShortcut(key, callback) {
    document.addEventListener("keydown", (event) => {
      if (event.key === key) {
        callback();
      }
    });
  }

  /**
   *
   * @param {string} key
   * @param {function(): void} callback
   */
  addCtrlShortcut(key, callback) {
    document.addEventListener("keydown", (event) => {
      if (event.ctrlKey && event.key === key) {
        callback();
      }
    });
  }
}
