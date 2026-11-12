import { createHash } from "node:crypto";
import { execFileSync as run } from "node:child_process";
import { mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, extname, join, relative, resolve, sep } from "node:path";

const EXPECTED_SOURCE_COMMIT = "f1a6fd6b3b14852a2a44be617b366cc4c0dbfd2a";
const SOURCE_REPOSITORY = "https://github.com/kolkrabeofdoom/die-stadt-die-den-dienstag-verkaufte";
const source = resolve(process.argv[2] ?? "../die-stadt-die-den-dienstag-verkaufte");
const projectRoot = resolve(import.meta.dirname, "..");
const staging = join(projectRoot, "artifacts", "dienstag-example");
const optimizedAssets = join(staging, "optimized-assets");
const snapshot = join(staging, "snapshot");
const destination = join(projectRoot, "examples", "die-stadt-die-den-dienstag-verkaufte.kraqel");
const crcTable = Array.from({ length: 256 }, (_, index) => {
  let value = index;
  for (let bit = 0; bit < 8; bit += 1) {
    value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  }
  return value >>> 0;
});

const sourceCommit = run("git", ["-C", source, "rev-parse", "HEAD"], { encoding: "utf8" }).trim();
if (sourceCommit !== EXPECTED_SOURCE_COMMIT) {
  throw new Error(`Unerwarteter Abenteuerstand: ${sourceCommit}`);
}
if (run("git", ["-C", source, "status", "--porcelain"], { encoding: "utf8" }).trim()) {
  throw new Error("Das Abenteuer-Repository enthält lokale Änderungen.");
}

rmSync(staging, { recursive: true, force: true });
mkdirSync(snapshot, { recursive: true });
run(
  "cargo",
  [
    "run",
    "-q",
    "-p",
    "kraqel-asset-service",
    "--bin",
    "optimize-example-images",
    "--",
    join(source, "companion-app", "app", "public", "assets"),
    optimizedAssets,
  ],
  { cwd: projectRoot, stdio: "inherit" },
);

const adventure = JSON.parse(
  readFileSync(join(source, "companion-app", "content", "v2.1-rc1", "adventure.json"), "utf8"),
);
const rights = readFileSync(join(source, "RECHTE.md"), "utf8").trim();
const uuid = (key) => {
  const bytes = createHash("sha256").update(`kraqel-dienstag:${key}`).digest().subarray(0, 16);
  bytes[6] = (bytes[6] & 0x0f) | 0x50;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const value = bytes.toString("hex");
  return `${value.slice(0, 8)}-${value.slice(8, 12)}-${value.slice(12, 16)}-${value.slice(16, 20)}-${value.slice(20)}`;
};
const paragraphDocument = (text) => ({
  version: 1,
  document: {
    type: "doc",
    content: String(text)
      .split(/\n{2,}/u)
      .map((paragraph) => ({
        type: "paragraph",
        content: paragraph.trim() ? [{ type: "text", text: paragraph.trim() }] : undefined,
      })),
  },
});
const document = (key, parentId, kind, title, position, plainText) => ({
  id: uuid(`document:${key}`),
  parentId,
  kind,
  title,
  position,
  revision: 0,
  content: paragraphDocument(plainText),
  plainText,
});
const chapter = (key, title, position) =>
  document(`chapter:${key}`, null, "chapter", title, position, "");

const introChapter = chapter("intro", "Über dieses Beispiel", 0);
const adventureChapter = chapter("adventure", "Das Abenteuer", 1);
const endingsChapter = chapter("endings", "Mögliche Enden", 2);
const referenceChapter = chapter("reference", "Schnellreferenz", 3);
const introText = `${adventure.subtitle}\n\n${adventure.premise}\n\nSicherheitsvereinbarungen\n${adventure.safety
  .map((item) => `• ${item}`)
  .join(
    "\n",
  )}\n\nDieses Projekt ist eine frei bearbeitbare Beispielkopie. Änderungen betreffen niemals die eingebettete Vorlage.`;
const introScene = document("intro", introChapter.id, "scene", "Willkommen in Veyra", 0, introText);
const rightsScene = document(
  "rights",
  introChapter.id,
  "scene",
  "Rechte, Quellen und Fanprojektstatus",
  1,
  `${rights}\n\nQuellrepository: ${SOURCE_REPOSITORY}\nQuellcommit: ${sourceCommit}\nInhaltsversion: ${adventure.content_version}`,
);
const sceneDocuments = adventure.scenes.map((scene, index) =>
  document(
    `scene:${scene.id}`,
    adventureChapter.id,
    "scene",
    `${scene.id} · ${scene.title}`,
    index,
    [
      `Ort: ${scene.location}`,
      `Dauer: ${scene.duration_minutes} Minuten`,
      `Ziel: ${scene.goal}`,
      `Auftakt\n${scene.opening}`,
      `Szenenverlauf\n${scene.beats}`,
      `Hinweise\n${scene.clues}`,
      `Spielleitungsbewegungen\n${scene.moves}`,
      `Schnitt\n${scene.cut}`,
    ].join("\n\n"),
  ),
);
const endingDocuments = adventure.endings.map((ending, index) =>
  document(
    `ending:${index}`,
    endingsChapter.id,
    "scene",
    ending.name,
    index,
    [`Auftakt: ${ending.opening}`, `Ergebnis: ${ending.outcome}`, `Preis: ${ending.cost}`].join(
      "\n\n",
    ),
  ),
);
const referenceScene = document(
  "quick-reference",
  referenceChapter.id,
  "scene",
  "Wenn … dann …",
  0,
  adventure.quick_reference.map((entry) => `Wenn ${entry.when}\nDann ${entry.then}`).join("\n\n"),
);
const documents = [
  introChapter,
  adventureChapter,
  endingsChapter,
  referenceChapter,
  introScene,
  rightsScene,
  ...sceneDocuments,
  ...endingDocuments,
  referenceScene,
];
const sceneDocumentByNumber = new Map(
  adventure.scenes.map((scene, index) => [Number(scene.id), sceneDocuments[index]]),
);

const entities = [];
const entityByKey = new Map();
const addEntity = (key, kind, name, properties) => {
  const entity = { id: uuid(`entity:${key}`), kind, name, properties };
  entities.push(entity);
  entityByKey.set(key, entity);
  return entity;
};
for (const npc of adventure.npcs) {
  addEntity(`npc:${npc.id}`, "character", npc.name, {
    Rolle: npc.role,
    Erscheinung: npc.look,
    Wunsch: npc.want,
    Befürchtung: npc.fear,
    Stimme: npc.voice,
    Geheimnis: npc.secret,
    Leitfrage: npc.question,
  });
}
for (const adversary of adventure.adversaries) {
  addEntity(`adversary:${adversary.id}`, "character", adversary.name, {
    Rolle: adversary.role,
    Schwierigkeit: String(adversary.difficulty),
    Schwellen: String(adversary.thresholds),
    Trefferpunkte: String(adversary.hp),
    Stress: String(adversary.stress),
    Angriff: adversary.attack,
    Schaden: adversary.damage,
    Reichweite: adversary.range,
    Motive: adversary.motives,
    Merkmale: adversary.features.join(" "),
  });
}
const veyra = addEntity("location:veyra", "location", "Veyra", {
  Beschreibung: "Eine regennasse Stadt, die den Dienstag verkauft hat.",
});
for (const location of [...new Set(adventure.scenes.map((scene) => scene.location))]) {
  addEntity(`location:${location}`, "location", location, { Stadt: "Veyra" });
}
const consortium = addEntity("faction:consortium", "faction", "Chronokonsortium", {
  Ziel: "Zeit-Zinsen dauerhaft kontrollieren",
});
const commonBeat = addEntity("faction:common-beat", "faction", "Gemeinsamer Takt", {
  Ziel: "Zeit als Gemeingut organisieren",
});
const cityWatch = addEntity("faction:watch", "faction", "Stadtwache", {
  Konflikt: "Ordnung, Verantwortung und der Schutz der Stadt",
});
for (const handout of adventure.handouts) {
  addEntity(`handout:${handout.id}`, "item", handout.title, {
    Nummer: String(handout.number),
    Beschreibung: handout.description,
  });
}

const relations = [];
const relation = (key, source, target, kind, properties = {}) =>
  relations.push({
    id: uuid(`relation:${key}`),
    sourceEntityId: source.id,
    targetEntityId: target.id,
    kind,
    properties,
  });
relation("common-against-consortium", commonBeat, consortium, "widersetzt sich");
relation("watch-protects-consortium", cityWatch, consortium, "schützt die Ordnung von");
for (const location of entities.filter(
  (entity) => entity.kind === "location" && entity !== veyra,
)) {
  relation(`location-in-veyra:${location.id}`, location, veyra, "liegt in");
}
for (const handout of adventure.handouts) {
  const item = entityByKey.get(`handout:${handout.id}`);
  const scene = adventure.scenes.find(
    (candidate) => Number(candidate.id) === Number(handout.scene),
  );
  if (scene)
    relation(
      `handout-location:${handout.id}`,
      item,
      entityByKey.get(`location:${scene.location}`),
      "erscheint in",
    );
}

const storyLinks = [];
const storyLinkKeys = new Set();
const linkStory = (documentId, entityId, role) => {
  const key = `${documentId}:${entityId}:${role}`;
  if (!storyLinkKeys.has(key)) {
    storyLinkKeys.add(key);
    storyLinks.push({ documentId, entityId, role });
  }
};
for (const [index, scene] of adventure.scenes.entries()) {
  const sceneDocument = sceneDocuments[index];
  linkStory(sceneDocument.id, entityByKey.get(`location:${scene.location}`).id, "Schauplatz");
  for (const entity of entities) {
    if (sceneDocument.plainText.includes(entity.name))
      linkStory(sceneDocument.id, entity.id, "Erwähnung");
  }
}
for (const handout of adventure.handouts) {
  const scene = sceneDocumentByNumber.get(Number(handout.scene));
  if (scene) linkStory(scene.id, entityByKey.get(`handout:${handout.id}`).id, "Handout");
}

const storyline = {
  id: uuid("storyline:return-tuesday"),
  name: "Die Rückkehr des Dienstags",
  color: "#7c5cff",
  position: 0,
};
const cards = sceneDocuments.map((scene, position) => ({
  documentId: scene.id,
  storylineId: storyline.id,
  povEntityId: null,
  status: "done",
  position,
}));

const mainTimeline = {
  id: uuid("timeline:two-evenings"),
  name: "Zwei Abende",
  mode: "relative",
  position: 0,
};
const oneShotTimeline = {
  id: uuid("timeline:one-shot"),
  name: "Vier-Stunden-One-Shot",
  mode: "relative",
  position: 1,
};
const countdownTimeline = {
  id: uuid("timeline:countdowns"),
  name: "Offene Countdowns",
  mode: "relative",
  position: 2,
};
const events = [];
const documentLinks = [];
const entityLinks = [];
const scheduleEvent = (timeline, entry, position, key) => {
  const event = {
    id: uuid(`event:${key}:${position}`),
    timelineId: timeline.id,
    title: entry.label,
    description: `${entry.scene}: ${entry.purpose}`,
    timeValue: null,
    position,
  };
  events.push(event);
  const match = String(entry.scene).match(/^(\d+)/u);
  if (match) {
    const scene = sceneDocumentByNumber.get(Number(match[1]));
    if (scene) documentLinks.push({ eventId: event.id, documentId: scene.id });
  }
};
adventure.schedule.forEach((entry, index) => scheduleEvent(mainTimeline, entry, index, "main"));
adventure.one_shot_schedule.forEach((entry, index) =>
  scheduleEvent(oneShotTimeline, entry, index, "one-shot"),
);
adventure.countdowns.forEach((countdown, position) => {
  const event = {
    id: uuid(`event:countdown:${countdown.id}`),
    timelineId: countdownTimeline.id,
    title: countdown.name,
    description: `Start: ${countdown.start}\nTicks: ${countdown.ticks}\nVoll: ${countdown.full}`,
    timeValue: null,
    position,
  };
  events.push(event);
});

const optimizedFiles = walkFiles(optimizedAssets);
const assetBySourcePath = new Map();
const assets = optimizedFiles.map((path) => {
  const relativePath = relative(optimizedAssets, path).split(sep).join("/");
  const bytes = readFileSync(path);
  const checksum = createHash("sha256").update(bytes).digest("hex");
  const dimensions = jpegDimensions(bytes);
  const sourcePath = `/assets/${relativePath.replace(/\.jpg$/u, ".png")}`;
  const actualSourcePath = sourcePath.replace(/handouts\/(.*)\.png$/u, "handouts/$1.jpg");
  const asset = {
    id: uuid(`asset:${relativePath}`),
    kind: "image",
    path: `assets/${checksum}.jpg`,
    originalName: basename(relativePath),
    title: titleFromFile(relativePath),
    mimeType: "image/jpeg",
    checksum,
    byteCount: bytes.length,
    width: dimensions.width,
    height: dimensions.height,
    provenance: "imported",
    tags: [relativePath.split("/")[0], "Beispielprojekt", "CC BY-SA 4.0"],
    links: [],
  };
  assetBySourcePath.set(actualSourcePath, asset);
  writeFile(join(snapshot, asset.path), bytes);
  return asset;
});
const linkAsset = (asset, targetKind, targetId, role) => {
  if (!asset) return;
  const key = `${targetKind}:${targetId}:${role}`;
  if (!asset.links.some((link) => `${link.targetKind}:${link.targetId}:${link.role}` === key)) {
    asset.links.push({ targetKind, targetId, role });
  }
};
linkAsset(
  assetBySourcePath.get("/assets/illustrationen/cover.png"),
  "document",
  introScene.id,
  "Cover",
);
for (const [index, scene] of adventure.scenes.entries()) {
  linkAsset(
    assetBySourcePath.get(scene.image),
    "document",
    sceneDocuments[index].id,
    "Szenenkarte",
  );
}
for (const npc of adventure.npcs) {
  linkAsset(
    assetBySourcePath.get(npc.portrait),
    "entity",
    entityByKey.get(`npc:${npc.id}`).id,
    "Porträt",
  );
}
for (const adversary of adventure.adversaries) {
  linkAsset(
    assetBySourcePath.get(adversary.portrait),
    "entity",
    entityByKey.get(`adversary:${adversary.id}`).id,
    "Porträt",
  );
}
for (const handout of adventure.handouts) {
  if (!handout.asset) continue;
  const asset = assetBySourcePath.get(handout.asset);
  const scene = sceneDocumentByNumber.get(Number(handout.scene));
  if (scene) linkAsset(asset, "document", scene.id, "Handout");
  linkAsset(asset, "entity", entityByKey.get(`handout:${handout.id}`).id, "Abbildung");
}
for (const asset of assets)
  asset.links.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b), "de"));

const profiles = [];
for (const npc of adventure.npcs) {
  const entity = entityByKey.get(`npc:${npc.id}`);
  profiles.push({
    entityId: entity.id,
    traits: { Erscheinung: npc.look, Rolle: npc.role },
    canonicalAssetId: assetBySourcePath.get(npc.portrait)?.id ?? null,
    exclusions: [],
  });
}
for (const adversary of adventure.adversaries) {
  const entity = entityByKey.get(`adversary:${adversary.id}`);
  profiles.push({
    entityId: entity.id,
    traits: { Rolle: adversary.role, Motive: adversary.motives },
    canonicalAssetId: assetBySourcePath.get(adversary.portrait)?.id ?? null,
    exclusions: [],
  });
}
const stylePreset = {
  id: uuid("style:veyra"),
  name: "Regennasses Veyra",
  visualRules: {
    Atmosphäre: "regennass, solidarisch, melancholisch-hoffnungsvoll",
    Palette: "Messing, Petrol, warmes Straßenbahnlicht",
    Technik: "Uhrwerke und kommunale Infrastruktur statt glatter Hochtechnologie",
  },
  applicableKinds: ["character", "location", "faction", "item"],
};
const promptEntities = [entityByKey.get("npc:mio").id, entityByKey.get("adversary:guard").id];
const promptDrafts = [
  {
    id: uuid("prompt:missing-morning"),
    name: "Der fehlende Morgen",
    sourceDocumentId: sceneDocuments[0].id,
    sourceDocumentRevision: 0,
    entityIds: promptEntities,
    stylePresetId: stylePreset.id,
    promptText:
      "Eine regennasse Straßenbahn hängt über einer Baugrube, während Mio Kest und solidarische Helfende den fehlenden Dienstag begreifen. Messing, Petrol, warmes Licht, keine heroische Gewaltpose.",
  },
];

const fixedFiles = [
  "manifest.json",
  "schema.json",
  "documents.json",
  "story-bible.json",
  "plotboard.json",
  "timelines.json",
  "visual-profiles.json",
  "prompt-drafts.json",
  "README.md",
];
const manifest = {
  format: "kraqel-project-snapshot",
  formatVersion: 3,
  project: {
    id: uuid("project:template"),
    name: "Die Stadt, die den Dienstag verkaufte (Beispiel)",
    schemaVersion: 1,
    settings: {
      example: {
        id: "die-stadt-die-den-dienstag-verkaufte",
        sourceRepository: SOURCE_REPOSITORY,
        sourceCommit,
        contentVersion: adventure.content_version,
        contentLicense: "CC BY-SA 4.0",
        rulesLicense: "DPCGL 2.0",
      },
    },
  },
  files: fixedFiles,
  assets,
};

writeJson("manifest.json", manifest);
writeJson("documents.json", { kind: "documents", documents });
writeJson("story-bible.json", { kind: "storyBible", entities, relations, links: storyLinks });
writeJson("plotboard.json", { kind: "plotboard", storylines: [storyline], cards });
writeJson("timelines.json", {
  kind: "timelines",
  timelines: [mainTimeline, oneShotTimeline, countdownTimeline],
  events,
  documentLinks,
  entityLinks,
});
writeJson("visual-profiles.json", {
  kind: "visualProfiles",
  profiles,
  stylePresets: [stylePreset],
});
writeJson("prompt-drafts.json", { kind: "imagePromptDrafts", drafts: promptDrafts });
writeFileSync(
  join(snapshot, "schema.json"),
  readFileSync(join(projectRoot, "docs", "data-format", "snapshot-v3.schema.json")),
);
writeFileSync(
  join(snapshot, "README.md"),
  `# KRAQEL-Beispielprojekt\n\nBearbeitbare Offline-Kopie von **Die Stadt, die den Dienstag verkaufte**.\n\nQuelle: ${SOURCE_REPOSITORY}\nQuellcommit: \`${sourceCommit}\`\nInhaltslizenz: CC BY-SA 4.0; Daggerheart-SRD-Anteile: DPCGL 2.0.\n`,
  "utf8",
);
for (const item of documents) {
  const level = item.kind === "chapter" ? "#" : "##";
  const text = `${level} ${item.title.replace(/[\r\n]/gu, " ")}\n\n${item.plainText.trimEnd()}\n`;
  writeFile(join(snapshot, "manuscript", `${item.id}.md`), Buffer.from(text, "utf8"));
}

const entries = walkFiles(snapshot).map((path) => ({
  name: relative(snapshot, path).split(sep).join("/"),
  bytes: readFileSync(path),
}));
entries.sort((a, b) => a.name.localeCompare(b.name, "en"));
mkdirSync(dirname(destination), { recursive: true });
writeFileSync(destination, createStoredZip(entries));
console.log(
  JSON.stringify(
    {
      destination,
      sourceCommit,
      documents: documents.length,
      entities: entities.length,
      timelines: 3,
      events: events.length,
      assets: assets.length,
      bytes: statSync(destination).size,
      sha256: createHash("sha256").update(readFileSync(destination)).digest("hex"),
    },
    null,
    2,
  ),
);

function writeJson(name, value) {
  writeFileSync(join(snapshot, name), `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function writeFile(path, bytes) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes);
}

function walkFiles(root) {
  const result = [];
  const visit = (directory) => {
    const entries = readdirSync(directory, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name, "en"),
    );
    for (const entry of entries) {
      const path = join(directory, entry.name);
      if (entry.isSymbolicLink()) throw new Error(`Symlink im Beispiel: ${path}`);
      if (entry.isDirectory()) visit(path);
      else if (entry.isFile()) result.push(path);
      else throw new Error(`Nicht regulärer Eintrag: ${path}`);
    }
  };
  visit(root);
  return result;
}

function titleFromFile(path) {
  return basename(path, extname(path))
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

function jpegDimensions(bytes) {
  if (bytes[0] !== 0xff || bytes[1] !== 0xd8) throw new Error("Kein JPEG.");
  let offset = 2;
  while (offset + 9 < bytes.length) {
    if (bytes[offset] !== 0xff) {
      offset += 1;
      continue;
    }
    const marker = bytes[offset + 1];
    const length = bytes.readUInt16BE(offset + 2);
    if (marker >= 0xc0 && marker <= 0xc3) {
      return { height: bytes.readUInt16BE(offset + 5), width: bytes.readUInt16BE(offset + 7) };
    }
    if (length < 2) throw new Error("Ungültiges JPEG-Segment.");
    offset += 2 + length;
  }
  throw new Error("JPEG-Abmessungen fehlen.");
}

function createStoredZip(entries) {
  const locals = [];
  const centrals = [];
  let offset = 0;
  for (const entry of entries) {
    const name = Buffer.from(entry.name, "utf8");
    const crc = crc32(entry.bytes);
    const local = Buffer.alloc(30 + name.length);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt16LE(0x0800, 6);
    local.writeUInt16LE(0, 8);
    local.writeUInt16LE(0, 10);
    local.writeUInt16LE(0x0021, 12);
    local.writeUInt32LE(crc, 14);
    local.writeUInt32LE(entry.bytes.length, 18);
    local.writeUInt32LE(entry.bytes.length, 22);
    local.writeUInt16LE(name.length, 26);
    name.copy(local, 30);
    locals.push(local, entry.bytes);

    const central = Buffer.alloc(46 + name.length);
    central.writeUInt32LE(0x02014b50, 0);
    central.writeUInt16LE(20, 4);
    central.writeUInt16LE(20, 6);
    central.writeUInt16LE(0x0800, 8);
    central.writeUInt16LE(0, 10);
    central.writeUInt16LE(0, 12);
    central.writeUInt16LE(0x0021, 14);
    central.writeUInt32LE(crc, 16);
    central.writeUInt32LE(entry.bytes.length, 20);
    central.writeUInt32LE(entry.bytes.length, 24);
    central.writeUInt16LE(name.length, 28);
    central.writeUInt32LE(offset, 42);
    name.copy(central, 46);
    centrals.push(central);
    offset += local.length + entry.bytes.length;
  }
  const centralDirectory = Buffer.concat(centrals);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10);
  end.writeUInt32LE(centralDirectory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, centralDirectory, end]);
}

function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}
