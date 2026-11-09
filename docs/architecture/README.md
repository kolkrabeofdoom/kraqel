# Architektur

KRAQEL trennt die React-Oberfläche von privilegierten Operationen. Typisierte Tauri-Commands bilden
die Grenze zu Rust. Rust verantwortet Dateisystem, SQLite, Git, Betriebssystem-Schlüsselspeicher und
alle externen Requests.

```text
React UI -> typisierte Commands -> Rust-Dienste -> SQLite / Projektordner / optionale Provider
```

SQLite ist während der Arbeit die autoritative lokale Datenquelle. Ein deterministischer Export
bildet den portablen, versionierbaren Zustand ab. Die laufende Datenbank wird nicht eingecheckt.

## Vertrauensgrenzen

1. **Projektordner:** grundsätzlich vertrauenswürdig, Pfade und importierte Dateien dennoch prüfen.
2. **Importe:** niemals als ausführbar behandeln; Typ, Größe und Inhalt validieren.
3. **Provider:** standardmäßig inaktiv; Kontext vor Übertragung anzeigen und explizit freigeben.
4. **Plugins/Editor-Inhalte:** als potenziell nicht vertrauenswürdig behandeln und Ausgaben säubern.

Architekturänderungen werden als ADR unter `decisions/` dokumentiert.
