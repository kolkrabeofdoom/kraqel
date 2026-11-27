# KQ-069: Lokale HMAC-Autorität statt DPAPI

16. November 2026. Der getrennte Forschungsbroker verwendet jetzt den
    [HMAC-Vertrag v2](../data-format/native-broker-recovery-v2.md).
    [Bereinigter Funktionsnachweis](native-hmac-broker.json). Die frühere
    [DPAPI-Crash-Serie](native-controller-crash.md) bleibt ein historischer eigener Nachweis.

## Tatsächliche Änderung und Verwahrung

DPAPI-Aufrufe und ihr Win32-Cryptography-Feature sind aus diesem Brokerpfad entfernt. Die
Domänenbackup/-wiederherstellungsroute wird dadurch nicht mehr für Wiederanlaufautorität verwendet.
Keine Registry-/Domänen-/Firewalländerung oder neue fachliche Netzausnahme. Ein einmaliger
32-Byte-Schlüssel aus dem fest betrachteten `getrandom`-/`ProcessPrng`-Backend bleibt in der
geschützten nativen Kontrollablage. RustCrypto-HMAC-SHA-256 authentisiert Magic, Version und
ursprüngliche Authoritybytes. Alte DPAPI-Daten haben keinen Fallback-/Migrationspfad.

Der Schlüssel ist lokal **unverschlüsselt**. Seine Geheimhaltung hängt an der privaten Windows-DACL,
regulären unverknüpften Dateien und der eingehaltenen Prozessgrenze. Ein Angriff mit denselben
Kontorechten, Administrationsrechten, Schlüsselkopie oder Prozessspeicherzugriff ist nicht gelöst.
Weder Hardware- noch zusätzliche Kontogeheimnisbindung behaupten. HMAC verschlüsselt auch den Beleg
nicht. Keine neuen Schlüssel oder Belege automatisch in einer benutzten Sitzung anlegen.

## Buildbindung und native Funktionsprüfung

Zusätzliche statische Prüfung der tatsächlichen Broker-PE-Importtabelle: `ProcessPrng` vorhanden,
keine `CryptProtectData`-/`CryptUnprotectData`-/DPAPI-NG-Protect-/Unprotect-Importe. Dieser Abgleich
ergänzt Code-/Dependencyprüfung und ersetzt weder ETW noch eine Attestierung aller dynamischen
Windows-Systemdienste.

Broker-Quellstand `b2e181134a9919f961188be1712bca9cae26c187`, SHA-256
`28fde769f00be412dcc755331f1a9424a39dac59e9a2f2de44dbc724e84825dd`. Die unveränderte App stammt aus
Quellstand `9d87e20d4a2526885c213bd186e4c22573d80744`, SHA-256
`4db1e731e6aee50fe01e9e368d1ded61ac61f3f9dc3e2238d509cee600098e1b`. Das Paar liegt separat unter
`%TEMP%\kraqel-kq069-hmac-broker-build-20261116-01`. Kein neuer Tauri-/Normalbuild.

Die native Serie mit diesem Paar besteht: sechs Controllerenden durch Selbstterminierung mit Code
86, zwölf neue Wiederanlaufprozesse und 30 App-Generationen. Alle bisherigen
Job-/SID-/Launch-/Hash-/Auftragsbindungen bleiben erhalten; kein neuer Importgrant. Je Crashpunkt 13
native Ablehnungen einschließlich fehlendem, verkürztem und falschem Schlüssel, insgesamt 78. Keine
automatische Schlüssel-/Journalneuanlage. Alle 30 suspendierten Root-Token geprüft, je sechs echte
Dateiöffnungen mit dem exakten App-Token verweigert, zusammen 180.

Unabhängige Nur-Lese-Prüfung mit Pythons HMAC-Implementierung bestätigt alle sechs tatsächlichen
Belege gegen ihre lokalen Schlüssel und beide Programmhashbindungen. Journal/SQLite/Receipt,
Unicode, Fremdschlüssel, leeres Staging und Kopienzahlen separat geprüft: vier unterbrochene
Aufträge ohne Kopie, zwei erfolgreiche mit je genau einer Kopie. Kein doppelter Import. 24 reguläre
Generationsberichte mit Code 77/leerem Job; für die sechs während des Controller- Kills beendeten
Seeds werden keine Abschlussberichte erfunden. Profile nach Stilllegung entfernt. Rohbelege,
Schlüssel, PIDs und lokale Sessionpfade bleiben lokal außerhalb Git.

`pnpm check`, 26 aktive Helfer-/Transporttests, Helfer-Clippy mit Warnungen als Fehler, Helferbuild
und 36 Python-Diagnosetests bestanden. Ein interaktiver Dialogtest bleibt bewusst ignoriert. Die
HMAC-Tests prüfen falsche Schlüssel, jedes veränderte Framebyte, jede Trunkierung, angehängte Bytes
und Altdaten. Der neue Auswertemodus `broker-ui` prüft verschachtelte Broker→App→WebView-Bäume mit
Positivkontrollen, Verlustzählern und vollständigen Enden; öffentliche Brokerereignisse oder
fehlende Enden/Kontrollen werden nicht zu Netzstille umgedeutet.

## Netz- und Produktgrenzen

Der anschließende [eigene frische ETW-Nachweis](native-hmac-broker-etw.md) ist gültig: Gesamtbaum
einschließlich negativer Brokerstarts, vier Kontrollen, Nullverluste, keine beobachteten
Netzereignisse außerhalb Loopback und 180 getrennte Loopback-Ereignisse. Kein alter App-/
Controllerbeleg wird auf dieses neue Paar übertragen. Die Funktionsaggregate beschreiben den
separaten vorausgehenden Lauf; Netzaggregate stehen in ihrer eigenen Datei. Keine neue Hörabnahme,
keine Produktfreigabe. Geschützte Produktablage, neue kombinierte Live-Aufträge, authentisierter
Update-/Migrationspfad und Produkt-/Packaging-Integration bleiben offen. Externe
COM-/Shell-/DNS-/Brokergrenzen bleiben gesondert nachzuweisen. Striktes Datenschutzziel unverändert;
Produkt-Q-01/Q-02 nicht bestanden.
