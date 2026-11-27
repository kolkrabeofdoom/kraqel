import { spawnSync } from "node:child_process";
import { chmodSync, existsSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const automatic = process.argv.includes("--auto");

function git(args, allowedStatuses = [0]) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
  if (result.error || !allowedStatuses.includes(result.status)) {
    throw new Error(result.error?.message ?? result.stderr.trim() ?? "Git-Aufruf fehlgeschlagen.");
  }
  return result.stdout.trim();
}

function install() {
  if (automatic && process.env.CI) return;
  if (!existsSync(resolve(root, ".git"))) {
    throw new Error("Kein Git-Checkout; Hooks können hier nicht eingerichtet werden.");
  }

  const configured = git(["config", "--get", "core.hooksPath"], [0, 1]);
  if (configured && configured !== ".githooks") {
    throw new Error("Eine andere core.hooksPath-Konfiguration ist aktiv; sie bleibt erhalten.");
  }
  if (!configured) {
    const previous = resolve(root, git(["rev-parse", "--git-path", "hooks"]));
    const existing = existsSync(previous)
      ? readdirSync(previous).filter((name) => !name.endsWith(".sample"))
      : [];
    if (existing.length) {
      throw new Error("Im bisherigen Hook-Ordner liegen eigene Hooks; sie bleiben erhalten.");
    }
  }

  chmodSync(resolve(root, ".githooks", "pre-push"), 0o755);
  git(["config", "--local", "core.hooksPath", ".githooks"]);
  console.log("KRAQEL: Push-Hook eingerichtet. Vor jedem Push läuft pnpm check.");
}

try {
  install();
} catch (error) {
  console.error(`KRAQEL: ${error.message}`);
  if (automatic) {
    console.error(
      "Hook-Einrichtung übersprungen. Für die manuelle Einrichtung: pnpm hooks:install",
    );
  } else {
    process.exitCode = 1;
  }
}
