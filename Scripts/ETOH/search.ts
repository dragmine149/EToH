// import { highlight_span } from "./usage";

// Code written by T3 Chat (AI assistant)
// File: tower-search.ts
// Purpose: custom tower search + highlight integration
// Notes:
// - Uses the user-provided `highlight_span(span, text, selected)` function
// - Keeps the supplied `isAcronymQuery` behaviour
// - Returns results with a numeric score and an array of reasons
// - Avoids `else` statements where possible (per user preference)

/**
 * Search result shape.
 */
export type SearchResult = {
  name: string;
  score: number;
  reasons: string[]; // all reasons that contributed (explanatory)
};

export let searched: Map<string, number> = new Map();
let timeout: number;

export function twodp(num: number): number {
  return ((num * 100) | 0) / 100;
}

/**
 * Shortens a tower name into the tower code (user function).
 * Example: "Citadel of Wacky Strategy" => "CoWS"
 */
export function shortTowerName(tower_name: string): string {
  return tower_name
    .split(/[\s-]/gm)
    .map((word) => word.toLowerCase())
    .map((word) => (word == "of" || word == "and" ? word[0] : word[0].toUpperCase()))
    .join("");
}

/**
 * Custom acronym detector (keeps the user's modified behaviour).
 * NOTE: This function intentionally uses the provided logic. Change
 * if you want a more general acronym detection.
 */
export function isAcronymQuery(q: string): boolean {
  // this might break in the future but for now its a good catch.
  if (q.length > 6) return false;

  let is_acro = q.startsWith("To") || q.startsWith("Co") || q.startsWith("So");
  let thirdLetter = q.charAt(2);
  // the original line was probably intended to assert something;
  // keep it as a no-op expression to preserve behaviour that was supplied.
  is_acro =
    is_acro &&
    thirdLetter === thirdLetter.toUpperCase() &&
    thirdLetter !== thirdLetter.toLowerCase();
  return is_acro;
}

export function improvedAcronymQuery(query: string, acros: string[]): 0 | 1 | 2 {
  if (query.length > 6) return 0;
  for (let acro of acros) {
    if (acro.startsWith(query)) return 1;
    if (acro.startsWith(query, 2)) return 2;
  }
  return 0;
}

/**
 * Check whether `sub` is a (case-insensitive) subsequence of `full`.
 */
export function isSubsequence(sub: string, full: string): boolean {
  let i = 0;
  let j = 0;
  for (; i < sub.length && j < full.length; j++) {
    if (sub[i] === full[j]) i++;
  }
  return i === sub.length;
}

/**
 * Levenshtein distance (case-insensitive).
 * Returns the number of letters required to change.
 */
export function levenshtein(a: string, b: string): number {
  const A = a.toLowerCase();
  const B = b.toLowerCase();
  const m = A.length;
  const n = B.length;
  const dp: number[][] = Array.from({ length: m + 1 }, () => new Array(n + 1).fill(0));
  for (let i = 0; i <= m; i++) dp[i][0] = i;
  for (let j = 0; j <= n; j++) dp[0][j] = j;
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      const cost = A[i - 1] === B[j - 1] ? 0 : 1;
      dp[i][j] = Math.min(
        dp[i - 1][j] + 1,
        dp[i][j - 1] + 1,
        dp[i - 1][j - 1] + cost
      );
    }
  }
  return dp[m][n];
}

/**
 * Fuzzy score between 0 and 100 based on normalised Levenshtein.
 */
export function fuzzyScore(a: string, b: string): number {
  const distance = levenshtein(a, b);
  const maxLen = Math.max(a.length, b.length, 1);
  return twodp(1 - distance / maxLen);
}

/**
 * Options for searchTowers.
 */
export type SearchOptions = {
  minScore?: number; // default 30
};

interface Names {
  name: string,
  short: string,
}

/**
 * Main search function.
 *
 * Behaviour:
 * - If `isAcronymQuery(query)` is true, scoring compares against the
 *   tower short codes produced by `shortTowerName`.
 * - Otherwise, scoring is performed against the full tower name.
 * - All matching "reasons" are collected in the `reasons` array for
 *   transparency/debugging.
 *
 * Returns results filtered by the configured minScore (default 30),
 * sorted by descending score and alphabetically as a tiebreaker.
 */
export function searchTowers(query: string, names: Names[], opts?: SearchOptions): SearchResult[] {
  opts = opts || {};
  const MIN_SCORE = typeof opts.minScore === "number" ? opts.minScore : 30;
  const q = query.trim().toLowerCase();

  if (q.length == 0) return names.map(({ name, short }) => { return { name, score: 100, reasons: ["empty query"] } });

  // const isAcr = isAcronymQuery(q);
  const qHasAcr = improvedAcronymQuery(q, names.map((v) => v.short));

  const scored = names.map(({ name, short }) => {
    let score = 100;
    const reasons: string[] = [];
    // const isAcr = improvedAcronymQuery(q, short);

    // small boost for previously searched.
    // let previous = (searched.get(name) ?? 1) / 40 * 0.05 + 1;
    // score *= previous;
    // reasons.push(`Previous: ${previous}`);

    if (qHasAcr) {
      // score *= ((1.5 / isAcr) + 1);
      // also includes boost.
      // reasons.push(`Acro boost: ${short}`);

      if (q == short) {
        score *= 3;
        reasons.push("Acro exact");
        return { name, score: Math.floor(score), reasons };
      }

      if (short.startsWith(q)) {
        score *= 1.4;
        reasons.push("Acro startswith");
      }

      const fs = fuzzyScore(q, short);
      if (fs > 0.6) {
        score *= (fs / 0.6) * 0.9;
        reasons.push(`acro fuzzy: ${fs}`);
      }
    }

    if (q == name) {
      score *= 3;
      reasons.push("Name exact");
      return { name, score: Math.floor(score), reasons };
    }

    if (name.split(" ")[0] == q.split(" ")[0]) {
      score *= 1.05;
      reasons.push("name is type");
    }

    let start = false;
    if (name.startsWith(q)) {
      score *= 2;
      reasons.push("name start");
      start = true;
    }

    // includes only works if it doesn't start with it.
    if (name.includes(q, start ? q.length : 0)) {
      score *= 1.15;
      reasons.push("name includes");
    }

    if (reasons.length == 0) {
      score *= 0.4;
      reasons.push("No reason, hence bad score");
    }

    // names are given bonus score if their length is shorter.
    let name_boost = twodp((1 / name.length) + 1);
    score *= name_boost;
    reasons.push(`Name boost: ${name_boost}`);

    // if (isAcr) {
    //   if (q == short) {
    //     score = score * 1;
    //     reasons.push("exact acronym");
    //   }
    //   if (shortUpper.startsWith(qUpper)) {
    //     score = Math.max(score, 80);
    //     reasons.push("acronym prefix");
    //   }
    //   if (isSubsequence(qUpper, shortUpper)) {
    //     score = Math.max(score, 60);
    //     reasons.push("acronym subsequence");
    //   }
    //   const fs = fuzzyScore(qUpper, shortUpper);
    //   const fsScore = Math.round(fs * 0.6); // downweight fuzzy for acronyms
    //   score = Math.max(score, fsScore);
    //   reasons.push(`acronym fuzzy:${fs}`);
    // }

    // const lowerName = name.toLowerCase();
    // const ql = q.toLowerCase();
    // if (lowerName === ql) {
    //   score = Math.max(score, 100);
    //   reasons.push("exact name");
    // }
    // if (lowerName.startsWith(ql)) {
    //   score = Math.max(score, 70);
    //   reasons.push("name startsWith");
    // }
    // if (lowerName.includes(ql)) {
    //   score = Math.max(score, 50);
    //   reasons.push("name includes");
    // }
    // const fs2 = fuzzyScore(ql, lowerName);
    // score = Math.max(score, fs2);
    // reasons.push(`name fuzzy:${fs2}`);

    // give a +10 boost to those which we have searched for before.
    // score += querySearch ? 10 : 0;

    return { name, score: Math.floor(score), reasons };
  });

  // if we have a perfect score.
  // clearTimeout(timeout);
  // timeout = setTimeout(() => {
  //   if (q.length == 0) return;
  //   scored.forEach((v) => {
  //     if (v.score == 0) return;
  //     searched.set(v.name, (searched.get(v.name) ?? 0) + 1)
  //   })
  // }, 1500);


  return scored
    .filter((r) => r.score >= MIN_SCORE)
    .sort((a, b) => {
      const byScore = b.score - a.score;
      return byScore !== 0 ? byScore : a.name.localeCompare(b.name);
    });
}

/**
 * Render helper.
 *
 * Creates a span element with the tower name as text and calls the
 * provided `highlight_span` (global) with `selected = false`.
 *
 * The highlight text is:
 * - the raw query when searching full names
 * - the query (acronym) when searching acronyms
 *
 * Note: `highlight_span` must be available on `window` and follow the
 * signature: highlight_span(span: HTMLSpanElement, text: string, selected: boolean)
 */
export function renderResultSpan(result: SearchResult, query: string): HTMLSpanElement {
  const span = document.createElement("span");
  // span.innerText = `${result.score} | ${result.name} | ${JSON.stringify(result.reasons)}`;

  const score = document.createElement("span");
  score.innerText = result.score.toString();
  const name = document.createElement("span");
  name.innerText = result.name;
  const reason = document.createElement("span");
  reason.innerText = result.reasons.join(", ");

  span.appendChild(score);
  span.appendChild(name);
  span.appendChild(reason);

  const textToHighlight = isAcronymQuery(query) ? query : query.trim();
  if (textToHighlight.length === 0) {
    return span;
  }

  highlight_span(name, textToHighlight, false);
  return span;
}
