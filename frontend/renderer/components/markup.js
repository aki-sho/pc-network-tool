export function escapeHtml(value) {
  return String(value ?? "—")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function createDetailList(items) {
  const rows = items.map(([label, value]) => `
    <div><dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value === "" ? "—" : value)}</dd></div>
  `);
  return `<dl class="detail-list">${rows.join("")}</dl>`;
}
