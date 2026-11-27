# Drehbuchdaten v2

Ein `EditorDocument` mit `version: 2` enthält genau `version` und `document`. Der Wurzelknoten `doc`
enthält mindestens einen `screenplayBlock`. Jeder Block hat genau eine Rolle unter `attrs.role` und
optional ein Array `content`. Leere Blöcke behalten ihre Rolle. Inline sind nichtleere Textknoten
und `hardBreak` zulässig. Textknoten dürfen je einmal `bold` und `italic` in `marks` tragen. Weitere
Attribute werden zurückgewiesen. Die sechs Rollen und die strengere Druckgrammatik stehen im
[Formatvertrag](../specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md). Ein unvollständiger Dialog
bleibt als Entwurf speicherbar. Prosa verwendet weiterhin v1.

Die Grenze von 5 MiB je gespeicherter Szenenhülle bleibt bestehen. Dieselbe Revisionsprüfung,
SQLite-Transaktion und FTS-Indizierung gelten für beide Formate. Mit der ersten gespeicherten
Drehbuchszene erhält das Projekt transaktional SQLite-`user_version = 7`, auch nach späterer
Rückwandlung in Prosa. Die Tabellen bleiben identisch. Ältere Anwendungen mit Schema 6 sperren das
Öffnen. Bestehende reine Prosaprojekte behalten Schema 6.

Load/Save v2 transportieren beide Editordokumentversionen. Die v1-Commands sind auf Prosa begrenzt.
Ein Snapshot mit Drehbuchdaten verwendet Manifest v4, mit denselben Dateien und Archivbudgets wie
v3. Maßgeblich für die Rollen ist `documents.json`; die begleitenden Markdown-Dateien bleiben
Klartextansichten. Ein v3-Manifest erlaubt ausschließlich Editordaten v1. Reine Prosaprojekte werden
weiterhin als Snapshot v3 ausgegeben.

## PDF-Ausgaben

Der eigenständige A4-Drehbuchdruck gibt die Rollenabsätze der ausdrücklich gewählten Drehbuchszenen
in Baumreihenfolge aus. Dieser Weg lässt Projekt-, Kapitel- und Szenentitel weg. In der nativen
Gliederung verwendet auch die einzelne gewählte Drehbuchszene dieses Profil.

Der anschließend erweiterte gewöhnliche Manuskriptdruck umfasst die bestätigte Auswahl
einschließlich ihrer Gliederungstitel. Kapitel und Manuskripte können reine Prosa, reine
Drehbuchszenen oder beides enthalten. Die bestehenden Druckregeln für Dialogfolge, Schriften und
Ausgabegrenzen gelten auch für die darin enthaltenen Drehbuchszenen. Ein Fehler in einem
ausgewählten Abschnitt bricht die Ausgabe vor der Veröffentlichung ab. Diese Erweiterung betrifft
den textbasierten PDF-Export. Der bebilderte Export und der kanonische DOCX-Austausch behalten ihre
eigenen Formatgrenzen.
