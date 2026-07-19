export function createStatusText(value) {
  if (value === true) return "Yes";
  if (value === false) return "No";
  return value ?? "-";
}