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
- [`overwrite.jsonc`](./overwrite.jsonc):  This is a file for listing some overwrites as for some badges, no matter what we do, we can't auto link.

## Update Log
This is seperate as it's not really a "release" and most people won't care how the data is structured.

### Data V2 (brief overview)
- better wiki parsing (like way way better)
- more data storage (so more information)
- ~90% automated (exact values... questionable), so i don't have to do stuff half the time.
