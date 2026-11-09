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
- kanonische Pfadprüfung, Größenlimits, Typprüfung und atomare Schreibvorgänge
- explizite Kontextvorschau und Freigabe vor jedem Cloudlauf
- Betriebssystem-Schlüsselspeicher statt Konfigurationsdateien
- deterministische Snapshots, Prüfsummen und getestete Wiederherstellung
- Abhängigkeitsprüfung und reproduzierbare CI

Dieses Dokument wird mit jeder neuen Vertrauensgrenze aktualisiert.
