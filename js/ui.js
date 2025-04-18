class Ui {
  constructor() {
    this.verbose = new Verbose("UI", '#110223');
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
  */
  updateLoadedUser(username) {
    if (!username) {
      username = 'No-one!';
    }

    document.querySelector('[id="viewing"] user').innerText = username;
  }

}

let ui = new Ui();
