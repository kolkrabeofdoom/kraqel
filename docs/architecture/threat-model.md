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
- Titeländerungen vergleichen den zuletzt angezeigten Titel; Baumumordnungen vergleichen alte und
  neue vollständige Geschwisterlisten. Löschen bindet Titel, Revision und bei Kapiteln die
  vollständige Kindliste und erfordert in der UI eine zweite Bestätigung
- Vorschlags-Diffs laden die Szene erneut, verlangen dieselbe Revision und besitzen keinen
  Schreibpfad; regelbasierte Kontinuitätsprüfungen bleiben vollständig lokal
- Bildimporte werden in Rust nach Signatur, Endung, Bytegröße, Pixelgrenzen und Dekodierbarkeit
  geprüft; Projektdateinamen entstehen nur aus UUID und Hash
- Backup und portable Snapshots kopieren ausschließlich datenbankreferenzierte Assetpfade und
  verifizieren Originale vor Veröffentlichung beziehungsweise Wiederherstellung gegen SHA-256
- visuelle Profile und Stilvorlagen sind projektgebunden, größenbegrenzt und werden nur nach
  erneuter Rust-Validierung gespeichert; ihre Commands besitzen keinen Netzwerkpfad
- Bildprompt-Entwürfe entstehen nur aus explizit gewählter Szene, visuellen Profilen und optionaler
  passender Stilvorlage; Szenentext wird als unvertrauenswürdiger Inhalt markiert, Quellrevisionen
  werden beim ersten Speichern geprüft und kein Prompt-Command besitzt einen Netzwerkpfad
- Bildfähigkeiten werden je Provider und Modell aus einer lokal validierten Registry gelesen;
  unbekannte Provider, Modelle, Fähigkeiten, Größen und Referenzmengen werden vor einem möglichen
  Provideraufruf abgelehnt
- Bildfreigaben enthalten exakt den angezeigten Prompt und die ausgewählten Referenzasset-UUIDs;
  eine Promptansicht oder ein gespeicherter Entwurf gilt nicht als Übertragungsfreigabe
- Generation-Job-Fehler verwenden ausschließlich feste Codes und statische Meldungen; Prompt,
  Providerantworten, Schlüssel, Pfade und fremde Diagnosedaten sind nicht Bestandteil des Fehlers
- Prompt-Textausgaben verwenden feste UUID-Dateinamen und bleiben unter `exports/prompts` im
  verwalteten Projekt; Snapshot v3 prüft ihre Dokument-, Profil- und Stilreferenzen
- Betriebssystem-Schlüsselspeicher statt Konfigurationsdateien
- feste offizielle HTTPS-Endpunkte für Cloudprovider; keine Redirects, Umgebungsproxies oder frei
  konfigurierbaren Produktionsziele
- höchstens zehn Minuten gültige Einmalfreigabe bindet Projekt, Dokumentrevision, Provider, Modell
  und Arbeitsauftrag und wird vor dem Cloudrequest verbraucht
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

Die Cloudadapter für OpenAI, Anthropic, Google und Mistral lesen Schlüssel ausschließlich innerhalb
der Rust-Grenze aus dem nativen Betriebssystemspeicher. Gespeicherte Werte werden nicht über IPC
zurückgegeben, in Projektdateien geschrieben oder geloggt. Beim erstmaligen Speichern existiert der
eingegebene Wert zwangsläufig kurz im React- und IPC-Speicher; danach wird er nur als
vorhanden/fehlend ausgewiesen. Der Provider sieht bei einer ausdrücklich ausgelösten Modellabfrage
bereits Konto und Netzwerkadresse, auch wenn noch keine Projektdaten übertragen werden.

Ein kompromittierter Betriebssystemspeicher, Root-/Administratorzugriff oder ein kompromittierter
Prozess kann Schlüssel und freigegebene Inhalte weiterhin lesen. KRAQEL kontrolliert außerdem weder
Kosten noch serverseitige Aufbewahrung, Trainingsnutzung oder Verfügbarkeit der Anbieter. Vor
produktiver Nutzung sind deren aktuelle Verträge und Datenschutzeinstellungen zu prüfen. Lokale
Protokolltests ersetzen keine echte Anbieterabnahme.

Die aktuelle Snapshotprüfung ist ausschließlich lesend und validiert den Formatvertrag. Eine
vollständige Wiederherstellung aus Snapshot ist noch nicht implementiert; ihre Datenverlust- und
Überschreibungsrisiken müssen vor einer schreibenden Importfunktion gesondert modelliert werden.

Die Bildprovider-Grenze besitzt in Version 0.1 nur Verträge, Registryvalidierung und einen lokalen
Test-Mock. Es gibt keinen produktiven Adapter, keinen Tauri-Command, keine Jobpersistenz und keine
Bildausgabe. Vor KQ-017 müssen Cloudfreigaben wie bei Text kurzlebig und einmalig gebunden,
Referenzdateien unmittelbar vor Übertragung erneut hashgeprüft, Ergebnisse nach Signatur und Größe
validiert sowie Kosten-, Lizenz- und Aufbewahrungsinformationen sichtbar gemacht werden.

Bilddecoder bleiben trotz Signatur-, Format-, Größen- und Pixelgrenzen eine Angriffsfläche für
präparierte Dateien. Deshalb ist `image` exakt und mit nur drei aktivierten Formaten gebunden und
muss regelmäßig aktualisiert werden. Der aktuelle WebView-Import überträgt außerdem das gesamte Bild
als Bytearray; die 25-MiB-Grenze gilt in Oberfläche und Rust, verhindert aber nicht jede kurzzeitige
Mehrfachbelegung beim Deserialisieren. Ein nativer Streamingimport ist vor wesentlich größeren
Einzeldateien oder Massenuploads neu zu bewerten.

Verbleibendes Risiko: Die kanonische Vorabprüfung ist nicht vollständig gegen ein gleichzeitiges
böswilliges Austauschen bereits geprüfter Verzeichnisse abgesichert. Vor der Unterstützung
unvertrauenswürdiger Projektwurzeln sind handle-relative Betriebssystemzugriffe zu bewerten.
