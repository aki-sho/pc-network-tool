export function initNavigation() {
  const links = [...document.querySelectorAll(".sidebar-nav a")];

  function updateActiveLink() {
    const active = links.find((link) => link.hash === window.location.hash) ?? links[0];
    for (const link of links) {
      if (link === active) {
        link.setAttribute("aria-current", "location");
      } else {
        link.removeAttribute("aria-current");
      }
    }
  }

  window.addEventListener("hashchange", updateActiveLink);
  updateActiveLink();
}
