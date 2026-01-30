# EToH TT Data Branch
Welcome to the Data branch of my EToH Tower Tracker.

## Purpose of this branch
Cleaner and easier access to any tower data. Also fully automated so that the data section rarley has to be touched (Thanks EToH WIKI Community)

## Why A separate branch?
Multiple benefits:
- It's easier for github actions
  Also means that they are out of the way and don't have any chance to break anything (not that they should). But getting an action to push to main is a bit annoying sometimes.
- Cleaner
  Although we could put everything in a folder, it's still a folder which rarley gets touched. So just tidies it up a bit

## Notes
This branch is not designed to be pushed into main. This will run alongside main, hence why we branched off at root (`000000`) instead of latest commit (`be0772cd` at time of split)

### Files of importance
- Anything in `BadgeUpdater`, this is where the program is.
- [`ignored.jsonc`](./ignored.jsonc):  This is a file for some badges which aren't worth showing. i might add in the future an option to show these on the UI, but thats a later issue.
- [`annoying_links.json`](./annoying_links.json):  This is a file for some badges to link them to some pages when fandom can't do it for us.
- [`overwrite.json`](./overwrite.json):  This is a file for listing some overwrites as for some badges, no matter what we do, we can't auto link.

### Reliance...
Most of this program is relaint on data being in a specific format, in a specific way. Numbers only existing once on a page and everything as we expect. This does mean we're more likely to break, but the wiki has been following
a pretty decent format system.

Then again, most things reliant on APIs, etc are well reliant on them. We're just taking a roundabout approach...

## Update Log
This is separate as it's not really a "release" and most people won't care how the data is structured. Additionally, it's more of an overview as trying to write out every single change is eh.. long
See the commit history for that...

### 0.2.1
- Reduced the amount of network requests from > 1000 (uncached) to like < 50 or so.
	- This does mean the cache doesn't get hit as often, but the cost of the requests is worth it.
	- This also uses the fandom API instead of pure `?action=raw`.

### 0.2.0 (aka DataV2)
- better wiki parsing (like way way better)
	- Is less prone to errors in changes of data.
	- Doesn't rely on python, hence not having to spin up python and take longer. Also means less lifetime nonsense.
- more data storage (so more information)
	- Now stores: `length`, `wiki link`. As well as extra info on events such as `event_name`, `event_area_name`, `event_items`. And a couple more fields.
		- *This does come at the downside of more data hence more network requests but its worth it.*
	- Data is all stored in one file instead of 2.
	- None-shrunken version of the data is less shrunk and way more readable.
- ~90% automated (exact values... questionable), so i don't have to do stuff half the time.
	- The left over ones are harder to automate or just not worth trying due to how inconesistant they are...
- Comments everywhere for documentation purposes (hopefully)
- Better logging for debugging purposes

## Todos?
- [ ] Make into workspaces? Basically split out [`wikitext`](./BadgeUpdater/src/wikitext) somehow.
- [ ] Tidy up the weird passing of `&[&Badge; 2]` all over the place to something easier to expand...
