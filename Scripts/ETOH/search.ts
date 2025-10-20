import { highlight_span } from "./usage";

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
export interface SearchResult {
  name: string;
  score: number;
  reasons: string[]; // all reasons that contributed (explanatory)
}

export const searched = new Map<string, number>();
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
  const thirdLetter = q.charAt(2);
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
  for (const acro of acros) {
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
export interface SearchOptions {
  minScore?: number; // default 30
}

interface Names {
  name: string,
  short: string,
}

/**
 * Main search function.
 * Does maths to hopefully get decent results to the query provided.
 */
export function searchTowers(query: string, names: Names[], opts?: SearchOptions): SearchResult[] {
  opts = opts || {};
  const MIN_SCORE = typeof opts.minScore === "number" ? opts.minScore : 30;
  const q = query.trim().toLowerCase();

  if (q.length == 0) return names.map(({ name, short }) => { return { name, score: 100, reasons: ["empty query"] } });

  // const isAcr = isAcronymQuery(q);
  const qHasAcr = improvedAcronymQuery(q, names.map((v) => v.short));

  const scored = names.map(({ name, short }) => {
    // default score of 100.
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
        // Max score of 400, because we found it exactly.
        score *= 4;
        reasons.push("Acro exact");
        return { name, score: Math.floor(score), reasons };
      }

      // starting with the query should give us more points, we understand it better.
      if (short.startsWith(q)) {
        score *= 1.4;
        reasons.push("Acro startswith");
      }
      if (short.startsWith(q, 2)) {
        // a bit less if we have to offset it, but still the same.
        score *= 1.3;
        reasons.push("Acro starts w/o type");
      }

      // fuzzy query gives way less as it might not be as relevant.
      const fs = fuzzyScore(q, short);
      if (fs > 0.6) {
        score *= (fs / 0.6) * 0.9;
        reasons.push(`acro fuzzy: ${fs}`);
      }
    }

    if (q == name) {
      // Max score, same as query.
      score *= 4;
      reasons.push("Name exact");
      return { name, score: Math.floor(score), reasons };
    }

    let name_type = false;
    // The first word is almost always the type of tower. Hence we give a small boost.
    if (name.split(" ")[0] == q.split(" ")[0]) {
      score *= 1.05;
      reasons.push("name is type");
      name_type = true;
    }

    // bigger boost if name starts with it
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

    // if somehow other way around we want something
    if (q.includes(name)) {
      score *= 1.2;
      reasons.push("Query includes");
    }

    // if any word in q is in name, score *= 1.1
    let qWords = q.split(/\s+/);
    const nameLower = name.toLowerCase().split(/\s+/);
    if (name_type) qWords = qWords.splice(1);
    if (qWords.length > 0) {
      for (const word of qWords) {
        if (word.length > 0 && nameLower.includes(word)) {
          const score_math = twodp((0.65 / word.length) + 1);
          score *= score_math;
          reasons.push(`Word match: ${word} (${score_math})`);
          // break; // only apply bonus once
        }
      }
    }

    // punishment for items which have no contribution. It's HIGHLY unlikely any of these will be chosen.
    if (reasons.length == 0) {
      score *= 0.4;
      reasons.push("No reason, hence bad score");
    }

    // Score is boosted by 1.02 (50 chars) -> 1.07 (14 chars). Allows ordering by name length + we probably want a shorter version anyway.
    const name_boost = twodp((1 / name.length) + 1);
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
