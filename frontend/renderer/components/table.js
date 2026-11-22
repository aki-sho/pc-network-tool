import { escapeHtml } from "./markup.js";

export function createTable(headers, rows, { label, emptyMessage, rowClass = () => "" }) {
  if (rows.length === 0) {
    return `<p class="empty-state">${escapeHtml(emptyMessage)}</p>`;
  }

  return `
    <div class="table-wrap" role="region" aria-label="${escapeHtml(label)}" tabindex="0">
      <table>
        <thead>
          <tr>
            ${headers.map((header) => `<th scope="col">${escapeHtml(header)}</th>`).join("")}
          </tr>
        </thead>
        <tbody>
          ${rows
            .map(
              (row, index) => `
                <tr class="${escapeHtml(rowClass(index))}">
                  ${row.map((cell) => `<td>${escapeHtml(cell)}</td>`).join("")}
                </tr>
              `
            )
            .join("")}
        </tbody>
      </table>
    </div>
  `;
}
