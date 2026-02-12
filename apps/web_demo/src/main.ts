import init, { clean_text } from "../pkg/link_cleaner_wasm";
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

  output.value = clean_text(input.value);
  setStatus("Bereinigt");
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
    setStatus("WASM bereit");
  } catch {
    setStatus("WASM konnte nicht geladen werden");
  }
}

cleanButton?.addEventListener("click", () => {
  void clean();
});

copyButton?.addEventListener("click", () => {
  void copyOutput();
});

void bootstrap();
