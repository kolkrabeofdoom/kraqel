// Browser-only UI adapter. Never bundled in a native build; no native transport claims.
type View = { phase: string; error: string | null; receipt: { documentCount: number } | null };
let current: View = { phase: "ready", error: null, receipt: null };
let consumed = false;
let pending: ((value: View) => void) | null = null;
export async function isolationCommand(command: string): Promise<unknown> {
  if (command === "isolation_test_status") return current;
  if (command === "isolation_test_import") {
    if (consumed) throw "ALREADY_CONSUMED";
    consumed = true;
    if (new URLSearchParams(location.search).get("isolationFailImport") === "1") {
      current = { phase: "failed", error: "IMPORT_FAILED", receipt: null };
      return current;
    }
    current = { phase: "awaiting", error: null, receipt: null };
    return new Promise<View>((resolve) => {
      pending = resolve;
    });
  }
  if (command === "isolation_test_continue") {
    current = { phase: "done", error: null, receipt: { documentCount: 2 } };
    pending?.(current);
    pending = null;
    return;
  }
  if (command === "isolation_test_cancel") {
    consumed = true;
    current = { phase: "cancelled", error: "CANCELLED", receipt: null };
    pending?.(current);
    pending = null;
    return;
  }
  throw "INVALID_TASK";
}
