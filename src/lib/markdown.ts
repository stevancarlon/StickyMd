import { marked, type Tokens } from "marked";
import DOMPurify from "dompurify";

// Track checkbox index across a single render pass
let checkboxIndex = 0;

const renderer: Partial<marked.Renderer> = {
  // Override list item rendering to replace checkbox inputs with styled spans.
  // This avoids all DOMPurify/disabled-input issues because we never emit
  // <input> elements at all.
  listitem(token: Tokens.ListItem): string {
    let itemHtml = this.parser.parse(token.tokens);

    if (token.task) {
      const idx = checkboxIndex++;
      const checkedClass = token.checked ? " checked" : "";
      const icon = token.checked ? "&#x2611;" : "&#x2610;";
      const checkbox =
        `<span class="md-checkbox${checkedClass}" data-checkbox-index="${idx}" role="checkbox" aria-checked="${token.checked}" tabindex="0">${icon}</span>`;
      // marked wraps the inner content in <p> for loose lists; strip the
      // leading <p> tag so the checkbox sits inside it cleanly.
      if (itemHtml.startsWith("<p>")) {
        itemHtml = `<p>${checkbox}${itemHtml.slice(3)}`;
      } else {
        itemHtml = `${checkbox}${itemHtml}`;
      }
      return `<li class="task-list-item">${itemHtml}</li>\n`;
    }

    return `<li>${itemHtml}</li>\n`;
  },
};

marked.use({ renderer, gfm: true, breaks: true });

export function renderMarkdown(source: string): string {
  // Reset the per-render checkbox counter
  checkboxIndex = 0;

  const raw = marked.parse(source) as string;

  return DOMPurify.sanitize(raw, {
    // We no longer need to allow <input>; we only need our custom
    // data-checkbox-index attribute and role/aria-checked/tabindex on spans.
    ADD_ATTR: ["data-checkbox-index", "role", "aria-checked", "tabindex"],
  });
}
