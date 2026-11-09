# ADR 0010: Feste Cloudprovider-Grenze und Betriebssystem-Schlüsselspeicher

Status: angenommen am 9. November 2026

## Kontext

F05 und KQ-011 verlangen OpenAI, Anthropic Claude, Google Gemini und Mistral mit Modellwahl,
Streaming, Schlüsselschutz und expliziter Kontextfreigabe. Cloudzugriffe dürfen den Offlinebetrieb
nicht voraussetzen und weder Schlüssel noch Manuskriptinhalte unbemerkt offenlegen.

## Entscheidung

Alle Cloudrequests entstehen ausschließlich im Rust-Dienst. Jeder Provider besitzt einen festen
offiziellen HTTPS-Endpunkt und einen eigenen Protokollparser. HTTP-Redirects und aus der Umgebung
übernommene Proxies sind deaktiviert. Produktionscode kann keine alternative Basisadresse setzen;
lokale HTTP-Endpunkte existieren nur innerhalb der Rust-Tests.

API-Schlüssel werden je Provider unter dem Dienstnamen `de.kraqel.desktop.ai` im nativen
Betriebssystem-Schlüsselspeicher abgelegt. React darf einen neuen Schlüssel über einen typisierten
Command setzen, erhält gespeicherte Werte aber niemals zurück. Der Rust-Prozess hält gelesene Werte
nur kurzzeitig in beim Freigeben überschriebenem Speicher und markiert Authentifizierungsheader als
sensibel.

Eine Freigabevorschau zeigt Ziel, Modell, Arbeitsauftrag, Szenentext und übertragene Metadaten. Rust
erzeugt dafür einen zufälligen, höchstens zehn Minuten gültigen Einmalbeleg im Arbeitsspeicher. Der
Beleg bindet Projekt, Dokument, Revision, Provider, Modell und Arbeitsauftrag, wird vor dem Request
verbraucht und kann weder verändert noch wiederverwendet werden.

## Folgen

- Modelllisten werden nur durch eine ausdrückliche Bedienhandlung abgerufen; dabei fließen keine
  Projektdaten, der Provider erkennt jedoch Konto und Netzwerkverbindung.
- Ohne gespeicherten Schlüssel und passenden Einmalbeleg entsteht kein Generierungsrequest.
- Providerantworten bleiben größenbegrenzte, verwerfbare Entwürfe mit Herkunftsangabe.
- Providerbedingungen, Kosten, serverseitige Aufbewahrung und Modellverfügbarkeit liegen außerhalb
  der technischen Kontrolle von KRAQEL und müssen vor produktiver Nutzung geprüft werden.
- Die Protokolle sind gegen lokale Testserver verifiziert; echte Cloudläufe benötigen eigene
  Schlüssel und bleiben eine manuelle Abnahme.
