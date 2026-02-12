import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

const input = document.querySelector<HTMLTextAreaElement>("#input");
const output = document.querySelector<HTMLTextAreaElement>("#output");
const copyButton = document.querySelector<HTMLButtonElement>("#copy");
const status = document.querySelector<HTMLParagraphElement>("#status");
let latestRequestId = 0;

function setStatus(message: string): void {
  if (status) {
    status.textContent = message;
  }
}

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
    if (status?.textContent === "Fehler beim Bereinigen") {
      setStatus("");
    }
  } catch {
    if (requestId !== latestRequestId) {
      return;
    }
    setStatus("Fehler beim Bereinigen");
  }
}

async function copyOutput(): Promise<void> {
  if (!output) {
    return;
  }

  try {
    await navigator.clipboard.writeText(output.value);
    setStatus("Output kopiert");
  } catch {
    setStatus("Kopieren fehlgeschlagen");
  }
}

input?.addEventListener("input", () => {
  void cleanLive();
});

copyButton?.addEventListener("click", () => {
  void copyOutput();
});

void cleanLive();
