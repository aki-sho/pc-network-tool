import { escapeHtml } from "./markup.js";

export function createCardGrid(items) {
  return `
    <div class="card-grid">
      ${items
        .map(
          ({ label, value, icon, note }) => `
            <div class="info-card">
              <p class="info-card-label">
                <svg class="icon" aria-hidden="true"><use href="./assets/ui-icons.svg#${escapeHtml(icon)}"></use></svg>
                ${escapeHtml(label)}
              </p>
              <p class="info-card-value${icon === "globe" ? " is-address" : ""}">${escapeHtml(value)}</p>
              <p class="info-card-note">${escapeHtml(note)}</p>
            </div>
          `
        )
        .join("")}
    </div>
  `;
}
