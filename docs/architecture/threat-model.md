# Bedrohungsmodell (Entwurf)

## Schutzgüter

- unveröffentlichte Manuskripte und Story-Bibel-Daten
- Bilder, Prompts, Metadaten und lokale Pfade
- API-Schlüssel und Provider-Konfiguration
- Integrität von Projekten, Backups und Git-Snapshots

## Relevante Bedrohungen

- Pfadtraversal, Symlink-Fluchten und Überschreiben außerhalb des Projektordners
- unbemerkte Cloudübertragung oder zu weit gefasste Kontextfreigabe
- Prompt-Injection aus importierten Dokumenten und Medienmetadaten
- bösartige oder übergroße Importdateien
- Schlüssel oder private Daten in Logs, Snapshots und Fehlerberichten
- manipulierte Projekte, Migrationen oder Abhängigkeiten

## Grundkontrollen

- minimale Tauri-Capabilities und sämtliche Dateizugriffe in Rust
- kanonische, komponentenweise Pfadprüfung ohne Symlinks; Größenlimits und Typprüfung bei Importen
- atomare Projekt-, Backup- und Recovery-Veröffentlichung innerhalb desselben Dateisystems
- explizite Kontextvorschau und Freigabe vor jedem Cloudlauf
- lokales Ollama nur über eine feste Loopback-Adresse; Systemproxies und HTTP-Redirects sind
  deaktiviert, Antworten und Anfragen besitzen Größenlimits
- Kontextfreigaben binden Dokument-ID und Revision; Änderungen zwischen Vorschau und Lauf führen zum
  Konflikt statt zur Übertragung eines anderen Textstands
- Betriebssystem-Schlüsselspeicher statt Konfigurationsdateien
- deterministische Snapshots, Prüfsummen und getestete Wiederherstellung
- deny-by-default Git-Pfadfreigabe; keine globale Git-Konfiguration, Hooks oder interaktiven Prompts
- Abhängigkeitsprüfung und reproduzierbare CI

Dieses Dokument wird mit jeder neuen Vertrauensgrenze aktualisiert.

Der Ollama-Adapter ist die erste aktive Providergrenze. Er sendet ausschließlich den unmittelbar
zuvor angezeigten Szenenklartext und den Arbeitsauftrag an `127.0.0.1`. Modellnamen, Providerfehler
und Streamereignisse werden bereinigt; Prompts und Antworten werden nicht geloggt oder automatisch
in kanonische Dokumente geschrieben. Ein lokal kompromittierter Ollama-Prozess kann den
freigegebenen Text dennoch lesen und bösartige beziehungsweise übergroße Antworten liefern; deshalb
bleiben Größenlimits, strikte JSONL-Validierung und manuelle Übernahme erforderlich.

Die aktuelle Snapshotprüfung ist ausschließlich lesend und validiert den Formatvertrag. Eine
vollständige Wiederherstellung aus Snapshot ist noch nicht implementiert; ihre Datenverlust- und
Überschreibungsrisiken müssen vor einer schreibenden Importfunktion gesondert modelliert werden.

Verbleibendes Risiko: Die kanonische Vorabprüfung ist nicht vollständig gegen ein gleichzeitiges
böswilliges Austauschen bereits geprüfter Verzeichnisse abgesichert. Vor der Unterstützung
unvertrauenswürdiger Projektwurzeln sind handle-relative Betriebssystemzugriffe zu bewerten.
