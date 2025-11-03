# ETOH
Tower tracker for the roblox game: Eternal-Towers-of-Hell (https://www.roblox.com/games/8562822414/Eternal-Towers-of-Hell)

This code is free to use for any user case, Just be sure to credit me.
Visit the site here: https://dragmine149.github.io/ETOH

## NOTICE
EToH Just recently had an update (monday). And as you can see, a lot of commits have been made since last update to this tracker. An update will come out soon. (I apologise for the delay, i am terrible at personal deadlines.)

## Planned features
- Switch over to [Cypress](https://docs.cypress.io/app/end-to-end-testing/writing-your-first-end-to-end-test) for testing and add more tests.
- Compare data between users. (Only after users have been downloaded)
- Graphs showing data (like difficulty jumps or similar)
- Get more data from github instead of having to update 2 repos when 1 thing changes.
- Improved filter system (with more options as to how to show data)
- Default / saved settings
- Better way of showing which badges are obtainable and which are not.
- Have towers sorted (locally) instead of relying on the server.
- Fully functional offline mode
- Ability to click on a badge to display badge information

## Cloning
This section is for the random person who wants to make their own. Well, go ahead, all i request is credit.

The main folder is `Scripts/Core`, this contains underlying classes where everything else is built uptop of. Without this, nothing works.

Following that, `Script/ETOHBridge` exists for some more important functions which do rely on EToH data a tad bit.

Lastly, `Scripts/ETOH` is where 90% of things related to EToH is stored. This also handles UI rendering due to it being customised FOR EToH.

The scripts outside, in `Scripts` are either important for debugging, running the code, or every single other script.
