function escapeHtml(value) {
  return String(value ?? "-")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function createCardGrid(items) {
  return `
    <div class="card-grid">
      ${items
        .map(
          ([label, value]) => `
            <div class="info-card">
              <div class="info-card-label">${escapeHtml(label)}</div>
              <div class="info-card-value">${escapeHtml(value)}</div>
            </div>
          `
        )
        .join("")}
    </div>
  `;
}