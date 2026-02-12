import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";

const input = document.querySelector<HTMLTextAreaElement>("#input");
const output = document.querySelector<HTMLTextAreaElement>("#output");
const copyOverlay = document.querySelector<HTMLButtonElement>("#copy-overlay");
const outputWrap = document.querySelector<HTMLDivElement>(".output-wrap");
const monitorToggle = document.querySelector<HTMLInputElement>("#monitor-enabled");
let latestRequestId = 0;
let copyFeedbackTimer: number | undefined;

type ClipboardCleanedPayload = {
  id: number;
  originalText: string;
  cleanedText: string;
  urlsModified: number;
  paramsRemoved: number;
};

let lastSeenClipboardCleanId = 0;

async function cleanLive(): Promise<void> {
  if (!input || !output) {
    return;
  }

  const requestId = ++latestRequestId;

  try {
    const cleaned = await invoke<string>("clean_text", { input: input.value });
    if (requestId !== latestRequestId) {
      return;
    }

    output.value = cleaned;
    syncCopyOverlayState();
  } catch {
    if (requestId !== latestRequestId) {
      return;
    }
  }
}

async function copyOutputFromOverlay(): Promise<void> {
  if (!output || !copyOverlay) {
    return;
  }

  try {
    await navigator.clipboard.writeText(output.value);
    copyOverlay.classList.add("is-copied");
    if (copyFeedbackTimer !== undefined) {
      window.clearTimeout(copyFeedbackTimer);
    }
    copyFeedbackTimer = window.setTimeout(() => {
      copyOverlay.classList.remove("is-copied");
    }, 1000);
  } catch {}
}

function syncCopyOverlayState(): void {
  if (!output || !outputWrap || !copyOverlay) {
    return;
  }

  const hasOutput = output.value.trim().length > 0;
  outputWrap.classList.toggle("has-output", hasOutput);
  copyOverlay.disabled = !hasOutput;
}

async function syncMonitorToggleState(): Promise<void> {
  if (!monitorToggle) {
    return;
  }

  try {
    const enabled = await invoke<boolean>("get_clipboard_monitor_enabled");
    monitorToggle.checked = enabled;
  } catch {
    monitorToggle.checked = true;
  }
}

async function setMonitorEnabled(enabled: boolean): Promise<void> {
  if (!monitorToggle) {
    return;
  }

  try {
    const applied = await invoke<boolean>("set_clipboard_monitor_enabled", { enabled });
    monitorToggle.checked = applied;
  } catch {
    monitorToggle.checked = !enabled;
  }
}

function applyClipboardCleaned(payload: ClipboardCleanedPayload): void {
  if (payload.id <= lastSeenClipboardCleanId) {
    return;
  }
  lastSeenClipboardCleanId = payload.id;

  if (input) {
    input.value = payload.originalText;
  }
  if (output) {
    output.value = payload.cleanedText;
    syncCopyOverlayState();
  }
}

async function syncLatestClipboardCleaned(): Promise<void> {
  try {
    const latest = await invoke<ClipboardCleanedPayload | null>("get_latest_clipboard_cleaned");
    if (latest) {
      applyClipboardCleaned(latest);
    }
  } catch {
    // Ignore transient query failures to avoid status flicker.
  }
}

input?.addEventListener("input", () => {
  void cleanLive();
});

copyOverlay?.addEventListener("click", () => {
  void copyOutputFromOverlay();
});

monitorToggle?.addEventListener("change", () => {
  void setMonitorEnabled(monitorToggle.checked);
});

void listen<ClipboardCleanedPayload>("clipboard-cleaned", (event) => {
  applyClipboardCleaned(event.payload);
});

void syncMonitorToggleState();
setInterval(() => {
  void syncLatestClipboardCleaned();
}, 900);
syncCopyOverlayState();
void cleanLive();
