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
- Betriebssystem-Schlüsselspeicher statt Konfigurationsdateien
- deterministische Snapshots, Prüfsummen und getestete Wiederherstellung
- Abhängigkeitsprüfung und reproduzierbare CI

Dieses Dokument wird mit jeder neuen Vertrauensgrenze aktualisiert.

Verbleibendes Risiko: Die kanonische Vorabprüfung ist nicht vollständig gegen ein gleichzeitiges
böswilliges Austauschen bereits geprüfter Verzeichnisse abgesichert. Vor der Unterstützung
unvertrauenswürdiger Projektwurzeln sind handle-relative Betriebssystemzugriffe zu bewerten.
