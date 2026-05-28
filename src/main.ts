import { invoke } from "@tauri-apps/api/tauri";

const urlInput = document.querySelector<HTMLInputElement>("#video-url");
const qualitySelect = document.querySelector<HTMLSelectElement>("#quality-select");
const statusEl = document.querySelector<HTMLElement>("#download-status");

async function downloadVideo(e?: Event) {
  e?.preventDefault();
  const url = urlInput?.value?.trim();
  const quality = qualitySelect?.value ?? "best";
  if (!url) {
    if (statusEl) statusEl.textContent = "Please enter a video URL.";
    return;
  }
  if (statusEl) statusEl.textContent = "Sending request...";
  try {
    const res = await invoke<string>("download_video", { url, quality });
    if (statusEl) statusEl.textContent = String(res);
  } catch (err) {
    if (statusEl) statusEl.textContent = `Error: ${err}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#download-form")?.addEventListener("submit", downloadVideo);
});
