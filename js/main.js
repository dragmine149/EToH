/**
* Shows an error message to the user
* @param {string} message The message to show
*/
function showError(message) {
  document.getElementById('error_message').innerText = message;
  document.getElementById('errors').hidden = false;
}
