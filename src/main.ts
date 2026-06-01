import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const urlInput = document.querySelector<HTMLInputElement>("#video-url");
const qualitySelect = document.querySelector<HTMLSelectElement>("#quality-select");
const statusEl = document.querySelector<HTMLElement>("#download-status");
const downloadBtn = document.querySelector<HTMLButtonElement>("#download-btn");
const cancelBtn = document.querySelector<HTMLButtonElement>("#cancel-btn");
const statusContainer = document.querySelector<HTMLElement>("#status-container");
const progressEl = document.querySelector<HTMLProgressElement>("#download-progress");

function validateURL(url: string): boolean {
  const youtubeRegex = /^(https?:\/\/)?(www\.)?(youtube|youtu|youtube-nocookie)\.(com|be)\//;
  return youtubeRegex.test(url);
}

async function downloadVideo(e: Event) {
  e.preventDefault();
  
  const url = urlInput?.value?.trim();
  const quality = qualitySelect?.value ?? "best";
  
  if (!url) {
    if (statusEl) statusEl.textContent = "❌ Please enter a video URL.";
    return;
  }
  
  if (!validateURL(url)) {
    if (statusEl) statusEl.textContent = "❌ Invalid YouTube URL.";
    return;
  }
  
  if (statusEl) statusEl.textContent = "⏳ Validating URL...";
  if (statusContainer) statusContainer.style.display = "block";
  if (downloadBtn) downloadBtn.disabled = true;
  
  try {
    const id = await invoke<string>("download_video", { url, quality });
    
    if (statusEl) statusEl.textContent = `📥 Download started...`;
    if (progressEl) { progressEl.value = 0; }
    if (cancelBtn) { cancelBtn.onclick = () => cancelDownload(id); }

    // listen for progress events
    const unlistenProgress = await listen<{ id: string; progress: string }>("download-progress", (event) => {
      const payload = event.payload;
      if (payload.id !== id) return;
      if (payload.progress) {
        const p = parseFloat(payload.progress);
        if (progressEl && !Number.isNaN(p)) progressEl.value = p;
        if (statusEl) statusEl.textContent = `📊 Progress: ${payload.progress}%`;
      }
    });

    const unlistenFinished = await listen<{ id: string; status: string }>("download-finished", (event) => {
      const payload = event.payload;
      if (payload.id !== id) return;
      if (statusEl) statusEl.textContent = `✅ Download completed!`;
      if (progressEl) progressEl.value = 100;
      if (downloadBtn) downloadBtn.disabled = false;
      unlistenProgress();
      unlistenFinished();
    });
    
    const unlistenError = await listen<{ id: string; error: string }>("download-error", (event) => {
      const payload = event.payload;
      if (payload.id !== id) return;
      if (statusEl) statusEl.textContent = `❌ Error: ${payload.error}`;
      if (downloadBtn) downloadBtn.disabled = false;
      unlistenError();
    });
  } catch (err) {
    if (statusEl) statusEl.textContent = `❌ Error: ${err}`;
    if (downloadBtn) downloadBtn.disabled = false;
  }
}

async function cancelDownload(id: string) {
  try {
    await invoke("cancel_download", { id });
    if (statusEl) statusEl.textContent = "⏹️ Download cancelled.";
    if (downloadBtn) downloadBtn.disabled = false;
  } catch (err) {
    if (statusEl) statusEl.textContent = `❌ Error cancelling: ${err}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector<HTMLFormElement>("#download-form")?.addEventListener("submit", downloadVideo);
});
