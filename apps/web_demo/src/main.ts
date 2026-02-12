import init, { clean_text } from "../pkg/link_cleaner_wasm";
import "./styles.css";

const input = document.querySelector<HTMLTextAreaElement>("#input");
const output = document.querySelector<HTMLTextAreaElement>("#output");
const copyButton = document.querySelector<HTMLButtonElement>("#copy");
const status = document.querySelector<HTMLParagraphElement>("#status");
let wasmReady = false;

function setStatus(message: string): void {
  if (status) {
    status.textContent = message;
  }
}

function cleanLive(): void {
  if (!input || !output) {
    return;
  }

  if (!wasmReady) {
    output.value = "";
    return;
  }

  output.value = clean_text(input.value);
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

async function bootstrap(): Promise<void> {
  try {
    await init();
    wasmReady = true;
    cleanLive();
    setStatus("WASM bereit");
  } catch {
    setStatus("WASM konnte nicht geladen werden");
  }
}

input?.addEventListener("input", () => {
  cleanLive();
});

copyButton?.addEventListener("click", () => {
  void copyOutput();
});

void bootstrap();
