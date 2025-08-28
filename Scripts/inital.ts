import { userManager } from "./Etoh";

document.addEventListener('DOMContentLoaded', () => {
  const url = new URL(location.toString());
  const user = url.searchParams.get("user");
  if (user) userManager.find_user(Number.isNaN(user) ? user : Number(user));
})
