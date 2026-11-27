import { spawnSync } from "node:child_process";

// Windows installs pnpm as a .cmd wrapper, which needs the Windows command interpreter.
// The command is fixed; remote names, URLs and refs are never passed to a shell.
const windows = process.platform === "win32";
const result = spawnSync(
  windows ? "cmd.exe" : "pnpm",
  windows ? ["/d", "/s", "/c", "pnpm check"] : ["check"],
  { stdio: ["ignore", "inherit", "inherit"] },
);

if (result.error) {
  console.error(`KRAQEL: Prüfung konnte nicht gestartet werden: ${result.error.message}`);
}
process.exitCode = result.status ?? 1;
