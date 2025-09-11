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
  "Citadel of Laptop Splitting",
  "Tower of Winning Every Run",
  "Tower of Up Is Down",
  "Citadel of Wacky Strategy",
  "Tower of Broken Promises",
  "Citadel of Gentle Winds",
  "Tower of Many Trials",
  "Citadel of Silent Echoes",
  "Tower of Shimmering Glass",
  "Tower of Eternal Night",
  "Citadel of Reckless Courage",
  "Tower of Falling Stars",
  "Citadel of Daring Solutions",
  "Tower of Forgotten Songs",
  "Tower of Rolling Thunder",
  "Citadel of Hidden Doors",
  "Tower of Endless Climb",
  "Citadel of Murky Depths",
  "Tower of Whispered Secrets",
  "Tower of Crystal Tears",
  "Citadel of Wacky Strategy Redux",
  "Tower of Winning Moments",
  "Citadel of Lofty Ambition",
  "Tower of Shattered Time",
  "Tower of Untold Stories",
  "Citadel of Lovable Chaos",
  "Tower of Clockwork Dreams",
  "Tower of Obsidian Light",
  "Citadel of Lost Maps",
  "Tower of Wind and Flame",
  "Citadel of Tiny Miracles",
  "Tower of Courageous Hearts",
  "Tower of Rising Courage",
  "Citadel of Wondrous Oddities",
  "Tower of Local Triumphs",
  "Tower of Wandering Souls",
  "Citadel of Quiet Resolve",
  "Tower of Witty Repartee",
  "Citadel of Patient Heroes"
];
console.log("search CoWS:", searchTowers("CoWS", data)); // default minScore 30
console.log("search Co (low threshold):", searchTowers("Co", data, { minScore: 10 }));
console.log("search 'Winning':", searchTowers("Winning", data));