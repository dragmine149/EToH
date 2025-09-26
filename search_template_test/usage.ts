import { searchTowers, renderResultSpan, demoData, data } from "./search";

/**
 * Highlights a span by creating span children. Uses `innerText` to avoid having to reget the text or do weird stuff.
 * @param span The span to affect.
 * @param text The text to highlight.
 * @param selected To include the optional class for selectedness.
 */
export function highlight_span(span: HTMLSpanElement, text: string, selected: boolean) {
  let regex = new RegExp(`[${text}]`, `gi`);
  span.innerHTML = span.innerText.replaceAll(regex, (match) => {
    return `<span class="highlight ${selected ? "selected" : ""}"">${match}</span>`;
  })
}

function update_ui() {
  const fragment = document.createDocumentFragment();

  searchTowers(query.value, data, { minScore: min.valueAsNumber }).forEach((result) => {
    let elm = renderResultSpan(result, query.value);
    fragment.appendChild(elm);
  })

  results.innerHTML = "";
  results.appendChild(fragment);
}

let query: HTMLInputElement;
let min: HTMLInputElement;
let minVal: HTMLSpanElement;
let results: HTMLDivElement;

document.addEventListener('DOMContentLoaded', () => {
  query = document.getElementById('query') as HTMLInputElement;
  min = document.getElementById('minScore') as HTMLInputElement;
  minVal = document.getElementById("minVal") as HTMLSpanElement;
  results = document.getElementById("results") as HTMLDivElement;

  query.addEventListener('input', (ev) => update_ui());

  min.addEventListener('input', (ev) => {
    let value = ((ev as InputEvent).target as HTMLInputElement).value;
    minVal.textContent = value;

    update_ui();
  });

  update_ui();

  minVal.textContent = min.value;
});
