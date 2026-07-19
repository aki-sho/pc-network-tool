import { appInfo } from "../../services/system/app-info.js";

export function renderAboutPage(root) {
  root.innerHTML = `
    <div class="about-box">
      <p><strong>${appInfo.name}</strong> v${appInfo.version}</p>
      <p>${appInfo.description}</p>
    </div>
  `;
}