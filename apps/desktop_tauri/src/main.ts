import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./styles.css";

const input = document.querySelector<HTMLDivElement>("#input");
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

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function normalizeText(text: string): string {
  return text.replace(/\r/g, "");
}

function getInputText(): string {
  if (!input) {
    return "";
  }

  return normalizeText(input.innerText);
}

function getCaretOffset(element: HTMLElement): number | null {
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0) {
    return null;
  }

  const range = selection.getRangeAt(0);
  if (!element.contains(range.startContainer)) {
    return null;
  }

  const preRange = range.cloneRange();
  preRange.selectNodeContents(element);
  preRange.setEnd(range.startContainer, range.startOffset);
  return preRange.toString().length;
}

function setCaretOffset(element: HTMLElement, offset: number): void {
  const selection = window.getSelection();
  if (!selection) {
    return;
  }

  const range = document.createRange();
  const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT);
  let remaining = Math.max(0, offset);
  let current: Node | null = walker.nextNode();

  while (current) {
    const len = current.textContent?.length ?? 0;
    if (remaining <= len) {
      range.setStart(current, remaining);
      range.collapse(true);
      selection.removeAllRanges();
      selection.addRange(range);
      return;
    }
    remaining -= len;
    current = walker.nextNode();
  }

  range.selectNodeContents(element);
  range.collapse(false);
  selection.removeAllRanges();
  selection.addRange(range);
}

function buildHighlightedHtml(original: string, cleaned: string): string {
  if (original.length === 0) {
    return "";
  }

  let i = 0;
  let j = 0;
  let html = "";
  let removedStart = -1;

  while (i < original.length) {
    if (j < cleaned.length && original[i] === cleaned[j]) {
      if (removedStart >= 0) {
        const removedChunk = original.slice(removedStart, i);
        html += `<span class="removed">${escapeHtml(removedChunk)}</span>`;
        removedStart = -1;
      }
      html += escapeHtml(original[i]);
      i += 1;
      j += 1;
      continue;
    }

    if (removedStart < 0) {
      removedStart = i;
    }
    i += 1;
  }

  if (removedStart >= 0) {
    const removedChunk = original.slice(removedStart);
    html += `<span class="removed">${escapeHtml(removedChunk)}</span>`;
  }

  return html.replace(/\n/g, "<br>");
}

function renderInput(original: string, cleaned: string): void {
  if (!input) {
    return;
  }

  const wasFocused = document.activeElement === input;
  const caretOffset = wasFocused ? getCaretOffset(input) : null;
  input.innerHTML = buildHighlightedHtml(original, cleaned);

  if (wasFocused && caretOffset !== null) {
    setCaretOffset(input, caretOffset);
  }
}

function setInputText(text: string, cleanedText: string): void {
  renderInput(text, cleanedText);
}

function focusInput(): void {
  if (!input) {
    return;
  }

  input.focus();
  setCaretOffset(input, getInputText().length);
}

async function cleanLive(): Promise<void> {
  if (!input || !output) {
    return;
  }

  const currentInput = getInputText();
  const requestId = ++latestRequestId;

  try {
    const cleaned = await invoke<string>("clean_text", { input: currentInput });
    if (requestId !== latestRequestId) {
      return;
    }

    renderInput(currentInput, cleaned);
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

  setInputText(payload.originalText, payload.cleanedText);
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
requestAnimationFrame(() => {
  focusInput();
});
void cleanLive();
