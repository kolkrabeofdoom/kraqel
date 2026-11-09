# ADR 0009: Lokales Ollama-Streaming mit revisionsgebundener Kontextfreigabe

Status: angenommen am 9. November 2026

## Kontext

F05 verlangt mehrere optionale Textprovider, Modellwahl, Streaming und Kontextfreigabe. KQ-010 führt
zunächst nur den lokalen Ollama-Adapter ein. Die Anwendung muss ohne Provider funktionieren, und
KI-Entwürfe dürfen kanonischen Text niemals automatisch verändern.

## Entscheidung

Textprovider implementieren eine asynchrone Rust-Grenze für Beschreibung, Modellabfrage und
geordnete Streamereignisse. Ollama wird ausschließlich über eine beim Bau geprüfte Loopback-Adresse
angesprochen. Systemproxies und Redirects sind deaktiviert; Browser und React erhalten keinen
direkten Netzwerkzugriff.

Vor einem Lauf lädt ein eigener Command den aktuellen Szenenklartext als Vorschau. Die Freigabe
referenziert Dokument-ID und Revision. Beim Start lädt Rust den Inhalt erneut und bricht bei einer
anderen Revision ab. Erst danach entsteht die Provideranfrage. Das Ergebnis wird über einen
typisierten Tauri-Kanal als verwerfbarer Entwurf übertragen und nie automatisch gespeichert.

## Folgen

- Ollama ist optional und wird nicht mit KRAQEL installiert.
- Fehlende Freigabe verhindert jeden Providerzugriff; ein lokaler Ausfall beeinträchtigt keine
  übrige Funktion.
- Größenlimits, strikte JSONL-Validierung und bereinigte Fehler begrenzen einen fehlerhaften lokalen
  Provider.
- Cloudprovider, Schlüsselspeicher, Laufpersistenz, Abbruch und manuelle Übernahme bleiben eigene
  Aufgaben.
