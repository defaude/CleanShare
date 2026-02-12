import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

const input = document.querySelector<HTMLTextAreaElement>("#input");
const output = document.querySelector<HTMLTextAreaElement>("#output");
const cleanButton = document.querySelector<HTMLButtonElement>("#clean");
const copyButton = document.querySelector<HTMLButtonElement>("#copy");
const status = document.querySelector<HTMLParagraphElement>("#status");

function setStatus(message: string): void {
  if (status) {
    status.textContent = message;
  }
}

async function clean(): Promise<void> {
  if (!input || !output) {
    return;
  }

  try {
    const cleaned = await invoke<string>("clean_text", { input: input.value });
    output.value = cleaned;
    setStatus("Bereinigt");
  } catch {
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

cleanButton?.addEventListener("click", () => {
  void clean();
});

copyButton?.addEventListener("click", () => {
  void copyOutput();
});
