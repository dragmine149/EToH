import { ui } from './ETOH/EtohUI';

/**
 * Helper function to load user based on the URL.
 * @param orig The place this got used from. Used for debugging purposes.
 */
function load_user_from_url(orig: string) {
  const url = new URL(location.toString());
  const user = url.searchParams.get("user");
  console.log(`attempting to load ${user} from ${orig}`);
  if (user) ui.load_user(user, true);
}

addEventListener('popstate', load_user_from_url.bind(this, "pop"))

// Console only function for debugging purposes. Separated out to reduce overhead + whatever.
globalThis.import_debug = async () => await import('./debug');
