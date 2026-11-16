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
let actionError: string | null = null;
let lastPhase: View["phase"] | null = null;
let flowStarted = false;
let preserveUserFocus = false;
let probePhase: View["phase"] | "unknown" = "unknown";
let probeQueue = Promise.resolve();
let probeCount = 0;
function probe(
  event:
    "action" | "phase" | "focus" | "blur" | "navigation" | "pointer" | "handoff" | "afterHandoff",
  trustedInput = false,
) {
  if (!flowStarted || probeCount >= 64) return;
  probeCount += 1;
  const active = document.activeElement;
  const focused =
    active === start
      ? "import"
      : active === proceed
        ? "continue"
        : active === cancel
          ? "cancel"
          : active === replay
            ? "replay"
            : active === document.body
              ? "body"
              : "other";
  const sample = {
    event,
    phase: probePhase,
    focused,
    hasFocus: document.hasFocus(),
    preserveUserFocus,
    trustedInput,
  };
  probeQueue = probeQueue
    .then(() => invoke<void>("isolation_test_focus_probe", { sample }))
    .catch(() => undefined);
}
document.addEventListener("focusin", (event) => probe("focus", event.isTrusted));
window.addEventListener("focus", () => probe("focus"));
window.addEventListener("blur", () => probe("blur"));
document.addEventListener("keydown", (event) => {
  if (
    [
      "Tab",
      "ArrowUp",
      "ArrowDown",
      "ArrowLeft",
      "ArrowRight",
      "Home",
      "End",
      "PageUp",
      "PageDown",
    ].includes(event.key)
  ) {
    preserveUserFocus = true;
    probe("navigation", event.isTrusted);
  }
});
document.addEventListener("pointerdown", (event) => {
  if (event.target instanceof Element && event.target.closest("button")) {
    preserveUserFocus = true;
    probe("pointer", event.isTrusted);
  }
});
const errors: Record<string, string> = {
  ALREADY_CONSUMED:
    "Dieser Auftrag ist bereits verbraucht. Es wurde kein zweites Projekt angelegt.",
  EXPIRED: "Der Auftrag ist abgelaufen. Der Import wurde verworfen.",
  CANCELLED: "Import abgebrochen. Es wurde keine Projektkopie angelegt.",
  INVALID_TASK: "Der Dateiauftrag ist ungültig.",
  IMPORT_FAILED: "Der Import konnte nicht abgeschlossen werden.",
  LENGTH_MISMATCH: "Die Dateigröße stimmt nicht mit der Freigabe überein.",
  READ_FAILED: "Die freigegebene Datei konnte nicht gelesen werden.",
  INTERRUPTED: "Der frühere Import wurde unterbrochen. Der Auftrag bleibt verbraucht.",
  STATE_UNAVAILABLE:
    "Der Auftragszustand konnte nicht sicher bestätigt werden. Der Auftrag bleibt gesperrt.",
};
function render(view: View) {
  probePhase = view.phase;
  const messages = {
    ready: "Ein einmaliger nativer Dateiauftrag ist bereit.",
    running: "Testarchiv wird geprüft …",
    awaiting: "Die Projektkopie ist vorbereitet. Import abschließen oder abbrechen.",
    done: `Projektkopie angelegt: ${view.receipt?.documentCount ?? 0} Dokumente.`,
    cancelled: "Import abgebrochen. Keine Projektkopie angelegt.",
    failed: "Der Import wurde nicht abgeschlossen.",
  };
  if (status.textContent !== messages[view.phase]) status.textContent = messages[view.phase];
  const message =
    actionError ?? (view.error ? (errors[view.error] ?? "Der Auftrag wurde abgelehnt.") : "");
  if (error.textContent !== message) error.textContent = message;
  start.disabled = view.phase !== "ready";
  proceed.disabled = view.phase !== "awaiting";
  cancel.disabled = !["ready", "running", "awaiting"].includes(view.phase);
  replay.disabled = !["done", "cancelled", "failed"].includes(view.phase);
  if (flowStarted && lastPhase !== view.phase) {
    const active = document.activeElement;
    const needsFocus =
      active === document.body ||
      !preserveUserFocus ||
      (active instanceof HTMLButtonElement && active.disabled);
    const target = view.phase === "running" ? cancel : view.phase === "awaiting" ? proceed : replay;
    if (needsFocus && !target.disabled) {
      probe("handoff");
      target.focus({ preventScroll: true });
      probe("afterHandoff");
    }
  }
  if (lastPhase !== view.phase) probe("phase");
  lastPhase = view.phase;
}
async function refresh() {
  try {
    render(await invoke<View>("isolation_test_status"));
  } catch {
    status.textContent = "Kein gültiger nativer Testkanal verfügbar.";
  }
}
async function action(command: string) {
  flowStarted = true;
  preserveUserFocus = false;
  probe("action");
  actionError = null;
  error.textContent = "";
  try {
    await invoke(command);
  } catch (reason) {
    actionError = errors[String(reason)] ?? "Der Auftrag wurde abgelehnt.";
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
