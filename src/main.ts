import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

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
    const id = await invoke<string>("download_video", { url, quality });
    if (statusEl) statusEl.textContent = `Download started (id: ${id})`;
    const progressEl = document.querySelector<HTMLProgressElement>("#download-progress");
    const cancelBtn = document.querySelector<HTMLButtonElement>("#cancel-btn");
    if (progressEl) { progressEl.style.display = "block"; progressEl.value = 0; }
    if (cancelBtn) { cancelBtn.style.display = "inline-block"; cancelBtn.onclick = () => { invoke("cancel_download", { id }); } }

    // listen for progress events
    const unlistenProgress = await listen("download-progress", (event) => {
      // @ts-ignore event.payload
      const payload = event.payload as any;
      if (payload.id !== id) return;
      if (payload.progress) {
        const p = parseFloat(payload.progress);
        if (progressEl && !Number.isNaN(p)) progressEl.value = p;
        if (statusEl) statusEl.textContent = `Progress: ${payload.progress}%`;
      }
    });

    const unlistenFinished = await listen("download-finished", (event) => {
      const payload = event.payload as any;
      if (payload.id !== id) return;
      if (statusEl) statusEl.textContent = `Finished`;
      if (progressEl) progressEl.style.display = "none";
      if (cancelBtn) cancelBtn.style.display = "none";
      unlistenProgress();
      unlistenFinished();
    });
  } catch (err) {
    if (statusEl) statusEl.textContent = `Error: ${err}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#download-form")?.addEventListener("submit", downloadVideo);
});
