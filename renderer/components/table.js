function escapeHtml(value) {
  return String(value ?? "-")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function renderCell(cell) {
  if (cell === null || typeof cell !== "object") return escapeHtml(cell);

  const description = cell.description
    ? `<span class="cell-description">${escapeHtml(cell.description)}</span>`
    : "";
  return `${escapeHtml(cell.text)}${description}`;
}

export function createTable(headers, rows) {
  return `
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            ${headers.map((header) => `<th scope="col">${renderCell(header)}</th>`).join("")}
          </tr>
        </thead>
        <tbody>
          ${rows
            .map(
              (row) => `
                <tr>
                  ${row.map((cell) => `<td>${renderCell(cell)}</td>`).join("")}
                </tr>
              `
            )
            .join("")}
        </tbody>
      </table>
    </div>
  `;
}
