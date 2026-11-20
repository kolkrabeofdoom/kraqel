# KQ-069: Frischer ETW-Netzvergleich des gesicherten v2-Prüfpaars

Stand: 16. November 2026. Gültiger neuer Netzbeleg für exakt das in der
[assistiven Wiederanlaufprüfung](native-assistive-restart.md) gesicherte v2-Paar.
[Bereinigte Aggregate](native-assistive-restart-etw.json). Runtime-Quellstand
`9d87e20d4a2526885c213bd186e4c22573d80744`, keine Runtimeänderung oder neuer Build für die Messung.

App-SHA-256: `4db1e731e6aee50fe01e9e368d1ded61ac61f3f9dc3e2238d509cee600098e1b`. Controller-SHA-256:
`64ddbcb277c38db75b6e324687d0122a62dcc4b4690e0fefe9249a027436007e`. Hashes vor/nach Erfassung sowie
an den tatsächlich in allen drei Containern ausgeführten Appkopien geprüft. Beide Programme stammen
aus der separaten lokalen Sicherung; der normale Debug-Build wurde nicht überschrieben.

## Messablauf und Ergebnis

Vorhandenes `scripts/m3-network/capture.ps1` mit dem vorhandenen PowerShell-7-Host und bestätigter
UAC-Erhöhung gestartet. Eigene begrenzte Sitzung, vier lokale TCP-/UDP-Kontrollen für IPv4/IPv6;
keine Firewall-/Loopback-Ausnahme-/Sicherheitsrichtlinienänderung oder physische Netztrennung.
Bereitschaft nach den bereits erfassten Kontrollen: `09:18:20.2857357 UTC`, Sitzungsende
`09:19:55.0114103 UTC`. Die Bereitschaftsangabe ist kein behaupteter exakter Providerstart.
ETL-Größe 4.128.768 Bytes. Nach allen Prozessenden wurde die Sitzung gestoppt und konvertiert.

Während derselben Messung:

| Ablauf                                    | App-Generationen | Ergebnis und Datenprüfung                                                                                                                             |
| ----------------------------------------- | ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Automatische v2-Serie                     | 28               | Sechs Crashpunkte, zweifacher Wiederanlauf/Altimportabwehr, vier abgewiesene Starts; vier unterbrochene und zwei abgeschlossene Aufträge, zwei Kopien |
| Manueller Wiederanlauf nach Unterbrechung | 2                | Replay und Schließen durch prüfende Person erledigt; native Abweisung, `interrupted`, null Kopien                                                     |
| Manueller Wiederanlauf nach Abschluss     | 2                | Replay und Schließen durch prüfende Person erledigt; native Abweisung, `succeeded`, genau eine Kopie                                                  |

Alle drei Controller bestanden mit Exitcode 0, leerem Job und entferntem Profil. Keine neue
Hörabnahme aus den Bedienbestätigungen dieser Netzserie ableiten; sie bleibt ein eigener
[vorheriger Nachweis](native-assistive-restart.md).

Der unveränderte Auswerter lief mit `--mode isolation-ui` und allen drei lokal festgehaltenen
Controller-PIDs. Gültige Messung: vier von vier Kontrollen, Nullverluste, keine Zuordnungslücken
oder unbekannten Ziele. **0 beobachtete Netzereignisse außerhalb Loopback im zugeordneten
Prozessbaum.** Separat je 88 WebView2-Loopback-Ereignisse der IDs 13 und 29, insgesamt 176.

Erfasst sind drei Controller-, 32 App-, 168 WebView- und 35 Console-Host-Lebenszeiten. Zusätzlich
zur Standardauswertung wurden die Appstarts pro Controller in Reihenfolge mit allen lokalen
Startbelegen abgeglichen: 32 passende App-Generationen, sämtliche 238 Prozessenden explizit
vorhanden. Jede der 28 GUI-Generationen hat sechs WebViews; die vier vor Fensteraufbau abgewiesenen
Starts haben keine. Kernel-Endereignisse bestätigen 26 erwartete harte Appenden mit Code 77, vier
Ablehnungen mit Code 2 und zwei reguläre manuelle Appenden mit Code 0. PID-Zuordnung erfolgt anhand
der Lebenszeiten, nicht als bloße Namens-/PID-Mengenprüfung. Alle 32 Root-Tokenprüfungen vor Resume
sind zusätzlich in den nativen Startbelegen bestätigt.

Unabhängige Nur-Lese-Prüfung nach Prozessende: erwartete Journalzustände und Kopienzahlen in allen
drei Ablagen, Journalintegrität `ok`, keine Zwischenreste. Veröffentlichte Kopien haben korrekte
Projekt-ID/Dokumentanzahl, exakten synthetischen Unicode-Inhalt, Receipt-/Archivhash und gültige
SQLite-Integrität/Fremdschlüssel. Keine Übernahme von Funktionsresultaten früherer Läufe.

## Vergleich, Grenzen und Übergabe

Der [gültige v1-Netzbeleg](native-ui-restart-etw.json) beobachtete ebenfalls 0 Nicht-Loopback-
Ereignisse, aber an einem anderen Paar und nur der automatischen Serie mit 28 Apps/144 WebViews.
Jetzt sind die neuen v2-Hashes und zusätzlich beide manuellen Pfade erfasst. Andere Programme,
Laufzeiten und Ereignispopulationen erlauben keinen Kausal- oder Ratenvergleich; keine Übertragung
alter Hashnachweise.

Roh-ETL/XML, PIDs, UUIDs und lokale Pfade bleiben lokal; nur datenarme Aggregate sind versioniert.
Erfassung und native Prozesse beendet, 32 Python-Diagnosetests bestanden. Dokumentationsänderung mit
Format-/Diff- und Fakten-/Hashprüfung, keine unnötigen Builds oder unveränderten Anwendungstests.
Das gesicherte Paar bleibt unter `%TEMP%\kraqel-kq069-assistive-restart-build-20261116-01`.

Transportmetadaten belegen keine HTTP-Bodies oder Upload-/Inhaltsfreiheit. Externe COM-/Shell-/
DNS-/Brokerpfade und kompromittierte Prozesse sind nicht vollständig kausal erfasst. Lokale
Messkontrollen ersetzen keine Kalibrierung sämtlicher öffentlicher Routen. Controller-Crash,
dauerhafter Produktbroker, kombinierter Live-/Neustartpfad und Produkt-/Packaging-Integration
bleiben offen. Normale Produktstarts bleiben unisoliert; #134 und Produkt-Q-01/Q-02 bleiben nicht
bestanden. Das strikte Datenschutzziel bleibt ohne fachliche Runtime-Ausnahme bestehen.
