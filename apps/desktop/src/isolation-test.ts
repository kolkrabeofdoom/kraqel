import { invoke } from "@tauri-apps/api/core";
import "./isolation-test.css";

type View = {
  phase: "ready" | "running" | "awaiting" | "done" | "cancelled" | "failed";
  error: string | null;
  receipt: { documentCount: number } | null;
};
const status = document.querySelector<HTMLParagraphElement>("#status")!;
const error = document.querySelector<HTMLParagraphElement>("#error")!;
const start = document.querySelector<HTMLButtonElement>("#import")!;
const proceed = document.querySelector<HTMLButtonElement>("#continue")!;
const cancel = document.querySelector<HTMLButtonElement>("#cancel")!;
const replay = document.querySelector<HTMLButtonElement>("#replay")!;
const errors: Record<string, string> = {
  ALREADY_CONSUMED:
    "Dieser Auftrag ist bereits verbraucht. Es wurde kein zweites Projekt angelegt.",
  EXPIRED: "Der Auftrag ist abgelaufen. Der Import wurde verworfen.",
  CANCELLED: "Import abgebrochen. Es wurde keine Projektkopie angelegt.",
  INVALID_TASK: "Der Dateiauftrag ist ungültig.",
  IMPORT_FAILED: "Der Import konnte nicht abgeschlossen werden.",
  LENGTH_MISMATCH: "Die Dateigröße stimmt nicht mit der Freigabe überein.",
  READ_FAILED: "Die freigegebene Datei konnte nicht gelesen werden.",
};
function render(view: View) {
  const messages = {
    ready: "Ein einmaliger nativer Dateiauftrag ist bereit.",
    running: "Testarchiv wird geprüft …",
    awaiting: "Die Projektkopie ist vorbereitet. Import abschließen oder abbrechen.",
    done: `Projektkopie angelegt: ${view.receipt?.documentCount ?? 0} Dokumente.`,
    cancelled: "Import abgebrochen. Keine Projektkopie angelegt.",
    failed: "Der Import wurde nicht abgeschlossen.",
  };
  if (status.textContent !== messages[view.phase]) status.textContent = messages[view.phase];
  if (view.error && error.textContent !== errors[view.error])
    error.textContent = errors[view.error] ?? "Der Auftrag wurde abgelehnt.";
  start.disabled = view.phase !== "ready";
  proceed.disabled = view.phase !== "awaiting";
  cancel.disabled = !["ready", "running", "awaiting"].includes(view.phase);
  replay.disabled = !["done", "cancelled", "failed"].includes(view.phase);
}
async function refresh() {
  try {
    render(await invoke<View>("isolation_test_status"));
  } catch {
    status.textContent = "Kein gültiger nativer Testkanal verfügbar.";
  }
}
async function action(command: string) {
  error.textContent = "";
  try {
    await invoke(command);
  } catch (reason) {
    error.textContent = errors[String(reason)] ?? "Der Auftrag wurde abgelehnt.";
  }
  await refresh();
}
start.addEventListener("click", () => {
  start.disabled = true;
  void action("isolation_test_import");
});
proceed.addEventListener("click", () => {
  void action("isolation_test_continue");
});
cancel.addEventListener("click", () => {
  void action("isolation_test_cancel");
});
replay.addEventListener("click", () => {
  void action("isolation_test_import");
});
void refresh();
window.setInterval(() => {
  void refresh();
}, 300);
