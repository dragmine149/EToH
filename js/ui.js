class Ui {
  constructor() {
    this.verbose = new Verbose("UI", '#110223');
  }

  updateSettings(setting_id, value) {
    if (setting_id.startsWith('verbose')) {
      localStorage.setItem(`setting-Debug-${setting_id}`, value);
    }
  }
  updateSettingsUI() {
    let checkboxes = document.querySelectorAll('.settings input[type="checkbox"]');
    checkboxes.forEach(checkbox => {
      if (!checkbox.id.startsWith("verbose")) {
        return;
      }

      checkbox.checked = localStorage.getItem(`setting-Debug-${checkbox.id}`) === 'true';
    });
  }

  /**
  * Shows an error message to the user
  * @param {string} message The message to show
  */
  showError(message) {
    document.getElementById('error_message').innerText = message;
    document.getElementById('errors').hidden = false;
  }

  hideError() {
    document.getElementById("errors").hidden = true;
  }

  updateLoadingStatus(text, output = false) {
    document.querySelector("[tag='status']").innerHTML = text;
    if (output) this.verbose.log(text);
  }

  /**
  * Update the UI to show that we are viewing that user.
  * @param {string} username The name of the user
  * @param {string} ui_name The ui made name to show (defaults to username value)
  */
  updateLoadedUser(username, ui_name = "") {
    if (ui_name == "" || ui_name == undefined) {
      ui_name = username;
    }

    this.verbose.log(`Updating UI to show ${ui_name} (${username})`);

    if (!username) {
      username = 'No-one!';
    }

    document.querySelector('[id="viewing"] user').innerText = ui_name;
    let url = new URL(location.href);
    url.searchParams.set('user', username);
    window.history.pushState({}, '', url);
    document.title = `${username} - EToH Tower Tracker`;
  }

  updateMainUi(visible) {
    document.getElementById('towers').hidden = !visible;
    document.getElementById('search').hidden = visible;
  }

}

let ui = new Ui();

document.addEventListener('DOMContentLoaded', () => {
  ui.updateSettingsUI();
});
