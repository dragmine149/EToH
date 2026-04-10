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

## Format
For those who want to use [shrunk.json](./shrunk.json), the format is a bit different than the typical [badges.json](./badges.json).

### Towers
Take this entry for example
```
A Simple Time,0,2125364300,1.25,0,2
```
This is split up into 6 sections as follows:
- name (Name of the tower w/o `Tower of` or `Citadel of`, etc..., where possible, NEATs do not include this seperation for example)
- old_badge_id (Badge id from before move. 0 = no id.
- new_badge_id (Badge id after move, every badge should have this.)
- difficulty (Difficulty of the tower in question)
- length (Length of the tower, as per [definitions.rs#L468-L484](https://github.com/dragmine149/EToH/blob/ac5d1d477dbcc1b97042eb55664c64bf862f99da/BadgeUpdater/src/definitions.rs#L468-L484). basically higher number = longer)
- type (Type of the tower, as per [definitions.rs#L376-L383](https://github.com/dragmine149/EToH/blob/ac5d1d477dbcc1b97042eb55664c64bf862f99da/BadgeUpdater/src/definitions.rs#L376-L383). Basically higher number = bigger tower (floor count))

For badges in [overwrite.jsonc](./overwrite.jsonc) and similar, they use this format.
```
First Easy,2124641807,2125419249
```
- name
- old_badge_id
- new_badge_id

No other information can be gathered from these badges, nor can the name be shrunk down even more.
Depending on the badge, the name might include `(TOWER ACRO)` to help hint to which one the badge is related to.

For badges linking to an item, they use this format
```
Present of Merrymaking Shrimp,0,2699038825006943,Shrimply Another Normal Tower, Amazingly
```
- name
- old_badge_id
- new_badge_id
- tower_name (optional, Even if there is no tower link, a trailing comma will still be included)

For more information about these badges, look up the coresponding tower name in the tower category.
Do note, these only support a direct tower link, items which require 2 or 4 towers for example, will not have any links.

### Random `#`?
*Due to not wanting to do complex stuff...* Any string with a `#` is our way of using a `,`. By compressing the data into a csv format, any tower names (*like **W**asn't **R**eally **A** **T**ower, **H**onestly*) will break the csv
as doing something like `.split(",")` will return one extra than it should. Whilst you can work around that, it's just easier if we say to replace with `#` instead.

### Modified Date **ALSO HAPPENS IN TOWERS.JSON**
The modified date is in seconds instead of milliseconds. Saves us 3 numbers of data as well it's not updated often enough to warrant the need for milliseconds.

### Difficultites 
Difficulties are a big cause of duplication. To reduce that we make use of bit shifting to store numbers. In total, a difficulty takes up 9 bits of any number. The bits are grouped into 3 sections:
- First 3 bits: The offset
- Second 3 bits: The first number
- Third 3 bits: The second number
```rs
fn process_difficulty(num: u16) -> (u8, u8, u8) {
	let offset = num >> 6;
	// Technically you don't 
	let first = (num - offset) >> 3;
	let second = num - offset - first;
	
	(offset as u8, first as u8, second as u8)
}
```

This system works because of HEAVY reliance on etoh being consistent. The following 3 rules have been found so far to work:
- For every area with difficulty requirements, the area will require X of one difficulty and X + Y of the previous difficulty. 
- No difficulty will require more than 7 towers
- No area requires towers in the `Easy` difficulty, always in difficulties higher than easy.

So for an example, the difficulty of `314` turns into `0000 0001 0011 1010` (`100 111 010`). which translates to:
- Offset of 4 as `100`, so the easiest difficulty (first) is intense.
- First tower is `111`, aka 7. Hence we have 7 towers for intense.
- Second tower is `010`, aka 2. Hence we have 2 towers for remorseless.

In other words, zone 10 area requirements.

Also see: https://github.com/dragmine149/EToH/blob/399328766d24a3ad82e3e1da7bf3b4bff1320612/BadgeUpdater/src/shrink_json_defs.rs#L420-L476 for the example of the whole deserializing process.

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
