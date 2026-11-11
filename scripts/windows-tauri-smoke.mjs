import { spawn } from "node:child_process";
import { access, readdir, realpath, rm } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import process from "node:process";

const ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf";
const TEMP_PREFIX = "kraqel-tauri-smoke";
const options = parseOptions(process.argv.slice(2));
const appBinary = resolve(options.app ?? "target/debug/kraqel-desktop.exe");
const tauriDriver = options.tauriDriver ?? process.env.KRAQEL_TAURI_DRIVER ?? "tauri-driver";
const edgeDriver = options.edgeDriver ?? process.env.KRAQEL_EDGE_DRIVER ?? "msedgedriver";
const isolatedDataRoot = resolve(options.dataRoot ?? join(tmpdir(), TEMP_PREFIX));

if (process.platform !== "win32") {
  throw new Error("Der Tauri-Smoke-Test ist auf Windows und Microsoft Edge WebDriver begrenzt.");
}
await access(appBinary);
await verifyExplicitExecutable(tauriDriver);
await verifyExplicitExecutable(edgeDriver);

await removeIsolatedDirectory(isolatedDataRoot);
const driverPort = await reservePort();
const nativePort = await reservePort();
const baseUrl = `http://127.0.0.1:${driverPort}`;
const driverLog = [];
const openSessions = new Set();
let driverProcess;
let driverStartError;

try {
  driverProcess = spawn(
    tauriDriver,
    [
      "--port",
      String(driverPort),
      "--native-port",
      String(nativePort),
      "--native-driver",
      edgeDriver,
    ],
    {
      env: process.env,
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    },
  );
  driverProcess.stdout.on("data", (chunk) => driverLog.push(String(chunk)));
  driverProcess.stderr.on("data", (chunk) => driverLog.push(String(chunk)));
  driverProcess.once("error", (error) => {
    driverStartError = error;
  });
  await waitForDriver(driverProcess, baseUrl, () => driverStartError);

  const projectName = `Tauri-Smoke-${Date.now()}`;
  const manuscriptText = "Der Kolkrabe speichert lokal. Die Teekanne verlangt einen Neustart.";

  const firstSession = await createSession(baseUrl, appBinary);
  openSessions.add(firstSession);
  await typeInto(firstSession, "css selector", "#project-name", projectName);
  await click(firstSession, "xpath", "//button[normalize-space()='Projekt anlegen']");
  await typeInto(
    firstSession,
    "css selector",
    "[role='textbox'][aria-label='Text von Szene 1']",
    manuscriptText,
  );
  await waitForElement(firstSession, "xpath", "//*[normalize-space()='Gespeichert']", 10_000);
  await deleteSession(firstSession);
  openSessions.delete(firstSession);

  const databases = await findFiles(isolatedDataRoot, "kraqel.sqlite3");
  if (databases.length !== 1) {
    throw new Error(
      `Erwartet wurde genau eine isolierte SQLite-Datei, gefunden: ${databases.length}.`,
    );
  }

  const secondSession = await createSession(baseUrl, appBinary);
  openSessions.add(secondSession);
  await click(secondSession, "xpath", `//button[.//strong[normalize-space()='${projectName}']]`);
  const editor = await waitForElement(
    secondSession,
    "css selector",
    "[role='textbox'][aria-label='Text von Szene 1']",
    10_000,
  );
  await waitForProperty(secondSession, editor, "innerText", manuscriptText, 10_000);
  await deleteSession(secondSession);
  openSessions.delete(secondSession);

  console.log("Windows-Tauri-Smoke-Test erfolgreich.");
  console.log(`Anwendung: ${appBinary}`);
  console.log("Nachweis: Start, IPC, Autosave, SQLite und Wiederöffnen.");
} catch (error) {
  if (driverLog.length > 0) {
    console.error("tauri-driver-Ausgabe:\n" + driverLog.join(""));
  }
  throw error;
} finally {
  for (const sessionId of openSessions) {
    await deleteSession(sessionId).catch(() => undefined);
  }
  if (driverProcess && driverProcess.exitCode === null) {
    driverProcess.kill();
  }
  await removeIsolatedDirectory(isolatedDataRoot);
}

function parseOptions(args) {
  const parsed = {};
  const normalizedArgs = args[0] === "--" ? args.slice(1) : args;
  const names = new Map([
    ["--app", "app"],
    ["--tauri-driver", "tauriDriver"],
    ["--edge-driver", "edgeDriver"],
    ["--data-root", "dataRoot"],
  ]);
  for (let index = 0; index < normalizedArgs.length; index += 2) {
    const property = names.get(normalizedArgs[index]);
    const value = normalizedArgs[index + 1];
    if (!property || !value) {
      throw new Error(
        "Verwendung: node scripts/windows-tauri-smoke.mjs [--app PFAD] [--tauri-driver PFAD] [--edge-driver PFAD] [--data-root PFAD]",
      );
    }
    parsed[property] = value;
  }
  return parsed;
}

async function verifyExplicitExecutable(command) {
  if (isAbsolute(command) || command.includes("/") || command.includes("\\")) {
    await access(resolve(command));
  }
}

async function reservePort() {
  return new Promise((resolvePort, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        server.close();
        reject(new Error("Es konnte kein lokaler Testport reserviert werden."));
        return;
      }
      server.close((error) => (error ? reject(error) : resolvePort(address.port)));
    });
  });
}

async function waitForDriver(child, url, startError) {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    if (startError()) throw startError();
    if (child.exitCode !== null) {
      throw new Error(`tauri-driver wurde vorzeitig mit Code ${child.exitCode} beendet.`);
    }
    try {
      const response = await fetch(`${url}/status`);
      if (response.ok) return;
    } catch {
      // Der lokale Treiber startet noch.
    }
    await delay(100);
  }
  throw new Error("tauri-driver war nicht rechtzeitig erreichbar.");
}

async function createSession(url, application) {
  const result = await webdriver(url, "POST", "/session", {
    capabilities: {
      alwaysMatch: {
        "tauri:options": { application },
      },
    },
  });
  const sessionId = result.sessionId ?? result.value?.sessionId;
  if (typeof sessionId !== "string") {
    throw new Error("tauri-driver lieferte keine Sitzungskennung.");
  }
  return { id: sessionId, url };
}

async function deleteSession(session) {
  await webdriver(session.url, "DELETE", `/session/${session.id}`);
}

async function typeInto(session, using, selector, text) {
  const element = await waitForElement(session, using, selector, 10_000);
  await sessionRequest(session, "POST", `/element/${element}/value`, { text });
}

async function click(session, using, selector) {
  const element = await waitForElement(session, using, selector, 10_000);
  await sessionRequest(session, "POST", `/element/${element}/click`, {});
}

async function waitForElement(session, using, selector, timeout) {
  const deadline = Date.now() + timeout;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const result = await sessionRequest(session, "POST", "/element", {
        using,
        value: selector,
      });
      const id = result.value?.[ELEMENT_KEY];
      if (typeof id === "string") return id;
    } catch (error) {
      lastError = error;
    }
    await delay(100);
  }
  throw new Error(`Element nicht gefunden (${selector}): ${lastError?.message ?? "Timeout"}`);
}

async function waitForProperty(session, element, property, expected, timeout) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const result = await sessionRequest(session, "GET", `/element/${element}/property/${property}`);
    if (result.value === expected) return;
    await delay(100);
  }
  throw new Error(`Eigenschaft ${property} enthielt nach dem Neustart nicht den erwarteten Text.`);
}

async function sessionRequest(session, method, path, body) {
  return webdriver(session.url, method, `/session/${session.id}${path}`, body);
}

async function webdriver(url, method, path, body) {
  const response = await fetch(`${url}${path}`, {
    method,
    headers: body === undefined ? undefined : { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const result = await response.json().catch(() => ({}));
  if (!response.ok || result.value?.error) {
    throw new Error(
      result.value?.message ?? `WebDriver-Anfrage scheiterte mit HTTP ${response.status}.`,
    );
  }
  return result;
}

async function findFiles(root, fileName) {
  const matches = [];
  for (const entry of await readdir(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) matches.push(...(await findFiles(path, fileName)));
    else if (entry.isFile() && entry.name === fileName) matches.push(path);
  }
  return matches;
}

async function removeIsolatedDirectory(directory) {
  const target = resolve(directory);
  const systemTemp = await realpath(resolve(tmpdir()));
  const canonicalTarget = join(await realpath(dirname(target)), basename(target));
  const relation = relative(systemTemp, canonicalTarget);
  if (
    !basename(target).startsWith(TEMP_PREFIX) ||
    relation === "" ||
    relation === ".." ||
    relation.startsWith(`..${sep}`) ||
    isAbsolute(relation)
  ) {
    throw new Error(`Unsicheres temporäres Löschziel abgelehnt: ${target}`);
  }
  await rm(target, { force: true, maxRetries: 5, recursive: true, retryDelay: 200 });
}

function delay(milliseconds) {
  return new Promise((resolveDelay) => setTimeout(resolveDelay, milliseconds));
}
