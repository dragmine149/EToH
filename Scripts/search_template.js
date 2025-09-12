// Code written by T3 Chat (AI assistant)
// JS: searchTowers updated to collect all reasons and apply min-score filtering

function shortTowerName(tower_name) {
  return tower_name
    .split(/[\s-]/gm)
    .map(function (word) {
      return word.toLowerCase();
    })
    .map(function (word) {
      return word == "of" || word == "and" ? word[0] : word[0].toUpperCase();
    })
    .join("");
}

// your modified acronym detector (kept as provided)
function isAcronymQuery(q) {
  let is_acro = q.startsWith("To") || q.startsWith("Co");
  let thirdLetter = q.charAt(2);
  is_acro ==
    is_acro &&
    thirdLetter === thirdLetter.toUpperCase() &&
    thirdLetter !== thirdLetter.toLowerCase();
  return is_acro;
}

function isSubsequence(sub, full) {
  var i = 0;
  var j = 0;
  for (; i < sub.length && j < full.length; j++) {
    if (sub[i].toLowerCase() === full[j].toLowerCase()) {
      i++;
    }
  }
  return i === sub.length;
}

function levenshtein(a, b) {
  var A = a.toLowerCase();
  var B = b.toLowerCase();
  var m = A.length;
  var n = B.length;
  var dp = new Array(m + 1);
  for (var i = 0; i <= m; i++) {
    dp[i] = new Array(n + 1).fill(0);
  }
  for (var ii = 0; ii <= m; ii++) dp[ii][0] = ii;
  for (var jj = 0; jj <= n; jj++) dp[0][jj] = jj;
  for (var i1 = 1; i1 <= m; i1++) {
    for (var j1 = 1; j1 <= n; j1++) {
      var cost = A[i1 - 1] === B[j1 - 1] ? 0 : 1;
      dp[i1][j1] = Math.min(
        dp[i1 - 1][j1] + 1,
        dp[i1][j1 - 1] + 1,
        dp[i1 - 1][j1 - 1] + cost
      );
    }
  }
  return dp[m][n];
}

function fuzzyScore(a, b) {
  var distance = levenshtein(a, b);
  var maxLen = Math.max(a.length, b.length, 1);
  return Math.max(0, Math.round((1 - distance / maxLen) * 100));
}

/**
 * searchTowers(query, data, opts)
 * opts:
 *  - minScore: number (default 30) minimum score to include result
 */
function searchTowers(query, data, opts) {
  opts = opts || {};
  var MIN_SCORE = typeof opts.minScore === "number" ? opts.minScore : 30;
  var q = query.trim();
  var isAcr = isAcronymQuery(q);
  var shortNames = data.map(function (d) {
    return { name: d, short: shortTowerName(d) };
  });
  var scored = shortNames.map(function (item) {
    var name = item.name;
    var short = item.short;
    var score = 0;
    var reasons = []; // collect all reasons
    if (isAcr) {
      var qUpper = q;
      var shortUpper = short;
      if (qUpper === shortUpper) {
        score = Math.max(score, 100);
        reasons.push("exact acronym");
      }
      if (shortUpper.startsWith(qUpper)) {
        score = Math.max(score, 80);
        reasons.push("acronym prefix");
      }
      if (isSubsequence(qUpper, shortUpper)) {
        score = Math.max(score, 60);
        reasons.push("acronym subsequence");
      }
      var fs = fuzzyScore(qUpper, shortUpper);
      var fsScore = Math.round(fs * 0.6);
      score = Math.max(score, fsScore);
      reasons.push("acronym fuzzy:" + fs);
    }
    if (!isAcr) {
      var lowerName = name.toLowerCase();
      var ql = q.toLowerCase();
      if (lowerName === ql) {
        score = Math.max(score, 100);
        reasons.push("exact name");
      }
      if (lowerName.startsWith(ql)) {
        score = Math.max(score, 70);
        reasons.push("name startsWith");
      }
      if (lowerName.includes(ql)) {
        score = Math.max(score, 50);
        reasons.push("name includes");
      }
      var fs2 = fuzzyScore(ql, lowerName);
      score = Math.max(score, fs2);
      reasons.push("name fuzzy:" + fs2);
    }
    return { name: name, score: score, reasons: reasons };
  });
  return scored
    .filter(function (r) {
      return r.score >= MIN_SCORE;
    })
    .sort(function (a, b) {
      return b.score - a.score || a.name.localeCompare(b.name);
    });
}

function renderResultSpan(result, query) {
  var span = document.createElement("span");
  span.innerText = result.name;
  var textToHighlight = isAcronymQuery(query) ? query : query.trim();
  if (textToHighlight.length === 0) {
    return span;
  }
  try {
    window.highlight_span(span, textToHighlight, false);
  } catch (e) {
    console.warn("highlight_span not found or threw an error:", e);
  }
  return span;
}

/* Test data and quick demo */
// Code written by T3 Chat (AI assistant)
// Placeholder tower list (ETOH/JTOH-style) for testing search behaviour

var data = [
  "Tower of Overcoming Hatred",
  "Tower of Inner and Outer Scaling",
  "This Is A Tower",
  "Wow! It's A Tower!",
  "Do Dat Look Like A Tower",
  "Tower of Modernistic Design Choices",
  "Tower of Genesis",
  "Tower of Buttons",
  "Tower of Motion Evolution",
  "Tower of Critical Damage",
  "Tower of Kinetic Potential",
  "Tower of No Strings Attached",
  "Tower of Keys To Success",
  "Citadel of Victory",
  "Tower of Dancing All Night",
  "Not Even A Tower",
  "Tower of A Simple Time",
  "Tower of Anger",
  "Tower of Madness",
  "Tower of Noticeable Infuriation",
  "Tower of Hecc",
  "Tower of Killjoys",
  "Tower of Keyboard Yeeting",
  "Tower of Stress",
  "Tower of Screen Punching",
  "Tower of Rage",
  "Tower of Impossible Expectations",
  "Citadel of Laptop Splitting",
  "Tower of True Skill",
  "Tower of Thanos Tower",
  "Maybe A Tower",
  "Tower of Phone Snapping",
  "Tower of Big Hole",
  "Tower of Cold Hands",
  "Tower of Falling and Failing",
  "Tower of Traps",
  "Tower of Deep Darkness",
  "Tower of Shattered Dreams",
  "Tower of Table Flipping",
  "Tower of Eternal Suffering",
  "Citadel of Wacky Strategy",
  "Tower of Difficulty Chart",
  "Tower of Funny Thoughts",
  "Tower of Inverted Colors",
  "Tower of Ancient Trickery",
  "Tower of Deep Sighing",
  "Tower of Fatness",
  "Tower of Winning Every Run",
  "Tower of Slight Inconvenience",
  "Tower of Wall Hugging",
  "Tower of Lotsa Damage",
  "Tower of Despair",
  "Citadel of Heights and Depths",
  "Tower of Confusion",
  "Tower of Spiralling Heights",
  "Tower of Getting Gnomed",
  "Tower of Terrible Mondays",
  "Tower of Elysium",
  "Tower of Linonophobia",
  "Tower of Leaning Ledges",
  "Tower of Dust and Decay",
  "Tower of Holy Flip",
  "Citadel of Uneasiness",
  "Tower of Oblivion",
  "Tower of Nonsensical Platforms",
  "Tower of Corrupted Nightmares",
  "Tower of Vivid Sections",
  "Tower of Inception",
  "Tower of Nice Views",
  "Tower of Trivial Resentment",
  "Tower of Rigid Success",
  "Tower of Downward Mobility",
  "Tower of Obvious Chaos",
  "Tower of Floral Fury",
  "Tower of Extraordinary Adventures",
  "Tower of Tokyo Heights",
  "Tower of Glitching and Healing",
  "Tower of Fractured Obstacles",
  "Citadel of Contrasting Regions",
  "Tower of Icy Blizzards",
  "Tower of Frightening Nightmares",
  "Tower of Friendly Jumps",
  "Tower of Environmental Pain",
  "Tower of Radiant Realms",
  "Tower of Twisted Inquisition",
  "Tower of A Depressing Future",
  "Tower of Intense Solar Chaos",
  "Tower of Dispersed Rooms",
  "Tower of Flustering Sections",
  "Tower of Niflheim",
  "Citadel of Scythe Recognition",
  "Tower of Strategic Mechanics",
  "Tower of Impossible Movement",
  "Tower of Rushed Building",
  "Tower of Collective Collaboration",
  "Tower of Feeling Lazy",
  "Tower of Distorted Aerodynamics",
  "Tower of Orientating Oscillating Opinions",
  "Tower of Never Giving Up, Ever",
  "Tower of Never Ending Dizziness",
  "Tower of Ultraviolet",
  "Citadel of True Exasperation",
  "Tower of Yearning Victory",
  "Tower of Extreme Hell",
  "Tower of Terrifying Beauty",
  "Tower of Elongated Runs",
  "Tower of Somewhat Simple Scaling",
  "Tower of Needing Basic Aptitude",
  "Tower of Voluminous Framework",
  "Tower of Vibrant Adventures",
  "Tower of Aamos' Anger",
  "Citadel of Constant Heart Stopping",
  "Tower of Uneasy Scaling",
  "Tower of Insanely Innovative Ideas",
  "Tower of Verdant Entropy",
  "Tower of Mean Tasks",
  "Tower of Suffering Outside",
  "Tower of Externalizing Insanity",
  "Tower of Generation Failure",
  "Tower of Great Overcomings",
  "Tower of Beat Block Berserk",
  "Tower of Questionable Trials",
  "Citadel of Lethargy",
  "Tower of Pastel Pillars",
  "Tower of High Adrenaline",
  "Tower of Bent Trauma",
  "Tower of Curved Ascent",
  "Tower of Frightening and Confusing Trials",
  "Tower of Bloodthirsty Kenos",
  "Tower of Journey's End",
  "Tower of Nervous Sweating",
  "Tower of Augmented Misery",
  "Tower of Champion's Road",
  "Tower of Cruel Punishment",
  "Steeple of Meaningless Decisions",
  "Tower of Jolly Good Fun",
  "Steeple of Low Woe",
  "Steeple of Pursuit",
  "This Is Probably A Tower",
  "Steeple of Uninstalling Roblox",
  "Tower of Immense Ire",
  "Steeple of Wall Punching",
  "Tower of Versatility",
  "Tower of Triangular Covering",
  "Steeple of Climbing",
  "Steeple of Huge Pain",
  "Tower of Increasing Stress",
  "Tower of Dangerous Expeditions",
  "Citadel of Weird Nostalgia",
  "Steeple of Towering Pillars",
  "Tower of Mind Breaking",
  "Steeple of Beginner's Journey",
  "Steeple of Devil's Snare",
  "Steeple of Infectious Foliage",
  "Steeple of Overgrowth",
  "Steeple of Nightfall",
  "Steeple of Overgrown Ascension",
  "Steeple of Greenhouse Placidity",
  "Steeple of Feudal Foliage",
  "Steeple of Lost In Quiescence",
  "Citadel of Biotech Genesis",
  "Steeple of Various Vivariums",
  "Steeple of Flourishing Wastelands",
  "Not Even A Flower",
  "Tower of Perilous Antipode",
  "Steeple of Buoyant Automations",
  "Steeple of Resort In Stasis",
  "Steeple of Ruinous Abate",
  "Tower of Ancestral Interference",
  "Steeple of Descendance",
  "Tower of There Is No Tower",
  "Steeple of Guiding Lights",
  "Steeple of Xenial Abyss",
  "Steeple of Wicked Grotto",
  "Tower of Shallow Waters",
  "Steeple of Involuntary Isolation",
  "Tower of Tenebrous Depths",
  "Steeple of Midnight Acropolis",
  "Steeple of Malignant Blight",
  "Tower of Excessive Weirdness",
  "Tower of Two Sided Troubles",
  "Citadel of Inside Situations",
  "Tower of Losing Our Zeal",
  "Tower of Diverging Layers",
  "Tower of Odd Odyssey",
  "Tower of Witchcraft and Wizardry",
  "Tower of Compact Spaces",
  "Tower of Tallying Every Mistake",
  "Tower of Triple Jeopardy",
  "Tower of Unsettling Heights",
  "Tower of Fruity Zeal",
  "Tower of Viaduct Traversing",
  "Tower of Corner Wedge Climbing",
  "Tower of Wanting More Time",
  "Tower of Shadowy Radiance",
  "Tower of Whisking It All",
  "Tower of Whirl of Winds",
  "Tower of Largely Limited Luminance",
  "Tower of Climactic Beats",
  "Tower of Rhythm Heaven",
  "Tower of Mechanical Disarray",
  "Tower of Wicked Fortress",
  "Tower of Lethean Recollection",
  "Tower of You're Late For Work",
  "Tower of Pursuit Led Astray",
  "Steeple of After-Life Detention",
  "Steeple of Balloon Commune",
  "Steeple of Two Minute Noodles",
  "Steeple of The Manufactured Monstrosity",
  "Tower of The Umbrella, The User",
  "Tower of Infiltrated Ruins",
  "Steeple of Itsy Bitsy",
  "Tower of A Disillusioned Existence",
  "Tower of Warranted Retribution",
  "Tower of Fleeing From Everything Ending",
  "Tower of House Without Home",
  "Steeple of Forlorn Blizzard",
  "Steeple of Cheesy Vengeance",
  "Tower of Zero Disturbances",
  "Tower of Peaceful Happiness and Tranquility",
  "Tower of Atlantic Depths",
  "Tower of Peace",
  "Tower of Hands Sweating",
  "Tower of Mirrored Hecc",
  "Tower of Contractual Obligations",
  "Tower of Yearning Success",
  "Tower of Absolute Vexation",
  "Tower of Wanting Extra Levels",
  "Tower of Ultimately Terrifying",
  "Tower of Extreme Dystopia",
  "Tower of Really Nasty Ideas",
  "Citadel of Peril",
  "Tower of Zespluz",
  "Tower of Thinning Layers",
  "Tower of Venomous Resonance",
  "Tower of Slanted Anticipation",
  "Tower of Ground Level Ascension",
  "Tower of Troublesome Adventures",
  "Tower of Unearthed Discoveries",
  "Tower of Dreams and Caverns",
  "Tower of Pleasant Fantasies",
  "Tower of Slipping and Sliding",
  "Citadel of Green Stuff",
  "Tower of Extraterrestrial Enchantment",
  "Tower of Arcanium Zenturing",
  "Tower of Triangular Terror",
  "Tower of Great Displeasure",
  "Tower of Chaotic Moments",
  "Tower of Double Trouble",
  "Tower of Six Feet Under",
  "Tower of Overestimating Difficulty",
  "Tower of Broken Bricks",
  "Tower of Client Object Frenzy",
  "Tower of Cracked, Crushed Cubes",
  "Tower of Xanthophobia",
  "Tower of Dimensional Hopping",
  "Tower of Mild Agitation",
  "Tower of Cogs and Steam",
  "Tower of Requiring Critical Help",
  "Citadel of Tricky Situations",
  "Tower of Menacing Creations",
  "Tower of Quirky Contraptions",
  "Tower of Infuriating Obstacles",
  "Tower of Cruel and Unusual Punishment",
  "Tower of Hopeless Hell",
  "Tower of Fairly Simple Challenges",
  "Tower of Aquatic Contemplation",
  "Tower of Fun and Simple Trials",
  "Tower of Up Is Down",
  "Tower of Wicked Wedges",
  "Tower of Elevator Travelling",
  "Tower of Pure Chroma",
  "Tower of Mechanically Induced Mayhem",
  "Tower of Shifting Slopes",
  "Tower of Dance Dance Destruction",
  "Tower of Funky Grooves",
  "Citadel of Quadrilaterals",
  "Tower of Tornado Vehemence",
  "Tower of Inevitable Failure",
  "Tower of Sovereign Traveling",
  "Tower of Super Stupid Security Systems",
  "Tower of Fractal Volution",
  "Tower of Bright Serenity",
  "Tower of Stairs To Spare",
  "Tower of Unhealthy Escalation",
  "Tower of Heiwana Kaze",
  "Tower of Horrifying Experiences",
  "Tower of Guided Trials",
  "Citadel of Corporate Enterprise",
  "Tower of Anticlimactic Outcomes",
  "Tower of Brutal and Bizarre Torment",
  "Tower of Zany Architecture",
  "Tower of Polychromatic Zero",
  "Tower of Alien Radiance",
  "Tower of Foreign Domains",
  "Tower of Rock Climbing",
  "Tower of Conflicting Frontiers",
  "Tower of Break It, Buy It",
  "Tower of Angled Scenery",
  "Tower of Hefty Magnitude",
  "Citadel of Twists and Weirdness",
  "Tower of Hazardous Laddering",
  "Tower of Watts",
  "Tower of Astral Fusion",
  "Tower of Massive Overheating",
  "Tower of Brief Challenges",
  "Tower of The Dripping Amalgam",
  "Tower of Micro Management",
  "Tower of Jazz",
  "Tower of Rough Waters",
  "Tower of Sweet Victory",
  "Tower of Bygone Relic",
  "Tower of Destructive Fever",
  "Tower of Growing Madness",
  "Tower of Cylindrical Extensions",
  "Tower of White Space",
  "Tower of Yore",
  "Tower of Cone Mesh Climbing",
  "Tower of Swift and Precise Sections",
  "Tower of Uncontrollable Ire",
  "Citadel of A Cruel Tale",
  "Tower of Fractured Memories",
  "Tower of Uncanny Agony",
  "Tower of Melons In Time",
  "Tower of Cosmic Tides",
  "Tower of A Tetromino Disaster",
  "Tower of X Madness",
  "Tower of Galactic Voyage",
  "Tower of Flame Restoration",
  "Tower of One Nine",
  "Tower of Never Ending Fun",
  "Tower of Manic Lows",
  "Tower of Overbearing Vertigo",
  "Citadel of Pyramid Escapades",
  "Tower of Time and Space Manipulation",
  "Tower of No Return",
  "Tower of Rough Endoplasmic Reticulum",
  "Tower of Astral Eclipse",
  "Tower of Descent Into Exile",
  "Tower of Arrantly Bodeful Cavern Depths",
  "Tower of Dusk To Dawn",
  "Tower of Latest Sensation",
  "Tower of Bonus Level",
  "Tower of Sugar Rush Deluxe",
  "Tower of Nothing New",
  "Tower of Furious Chicken Brawl",
  "Tower of Falling Up",
  "Tower of Eye Candy",
  "Tower of Big Blocky Beefy Buttons",
  "Tower of Minus Facility",
  "Citadel of Increasing Claustrophobia",
  "Tower of Voyaging Into The Earth",
  "Tower of Hollow Reformations",
  "Tower of Panelling Barricades",
  "Tower of Wildly Wacky Wonders",
  "Tower of Empty Meaningless Patterns",
  "Tower of Clandestine Zones",
  "Tower of Resonant Landscapes",
  "Tower of Interstellar Terrarium",
  "Tower of Valiant Verges",
  "Tower of Expecting The Unexpected",
  "Tower of Mach Nine",
  "Tower of One Two Three Four",
  "Tower of The Despondent Fortress",
  "Tower of Prismatic Instability",
  "Tower of Power Laws",
  "Tower of Rain on My World",
  "Tower of Tee Hee Time",
  "Tower of Blast Power",
  "Tower of Complexity and Volatility",
  "Tower of Raging Tempest",
  "Steeple of Biome Traversing",
  "Steeple of Astounding Sorcery",
  "Steeple of Realm Odyssey",
  "Steeple of Atmospheric Powers",
  "Steeple of Twisted Crystals",
  "Steeple of Magical Elements",
  "Steeple of Crystal Ascension",
  "Steeple of Witch Calamity",
  "Steeple of Magical Collaborations",
  "Tower of Icy Adventures",
  "Possibly A Tower",
  "Steeple of Mystical Marine",
  "Steeple of Abyssal Upturn",
  "Steeple of Sunny Island Shenanigans",
  "Steeple of Aquatic Rallies",
  "Steeple of Hallucinatory Spectacles",
  "Steeple of Desolate Isles",
  "Tower of Cyclonic Isles",
  "Steeple of Upright Cliffs",
  "Steeple of Cliffside Falls",
  "Tower of Traversing The Tropics",
  "Steeple of Underlying Breezes",
  "Steeple of Dusty Dunes",
  "Tower of Annoyingly Simple Trials",
  "Tower of Vesi Leikki",
  "Tower of Another Beginning",
  "Tower of Autumn Harvest",
  "Tower of Insult To Injury",
  "Tower of Generation Retro",
  "Tower of Quick, Brown Fox!",
  "Tower of Sparkling Rainbow Water",
  "Tower of One Equals Zero",
];
console.log("search CoWS:", searchTowers("CoWS", data)); // default minScore 30
console.log("search Co (low threshold):", searchTowers("Co", data, { minScore: 10 }));
console.log("search 'Winning':", searchTowers("Winning", data));
