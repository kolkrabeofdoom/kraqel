# ADR 0034: Windows-Laufzeitgrenze nach WebView2-Netzbefund

Status: **Entscheidungsvorlage, nicht angenommen.** KQ-069 / Issue #134. 15. November 2026. Keine
Änderung des Pflichtenhefts und keine Netzfreigabe.

## Belegter Konflikt

Pflichtenheft Revision 1 verlangt Offlinefähigkeit, keine Telemetrie ab Werk und keine Übertragung
von Manuskripttexten, Bildern oder Metadaten bei Cloud-Opt-out (§1, §5, §10). KQ-068 misst
öffentliche WebView2-Verbindungen. KQ-069 reproduziert Konfigurations- und Komponentenupdate-Starts
bereits ohne Projekte. Die unterstützten betrachteten APIs ergeben keinen belegten universellen
Ausschalter. Siehe [Versuch, Quellen und Grenzen](../../testing/native-m3-webview-network.md).

Eine funktionierende Offline-App und fehlender eigener Telemetriecode erfüllen noch keine
vollständige Windows-Laufzeit-Netzstille. Wir nehmen weder die Unbedenklichkeit aller Bodies noch
eine garantierte Übertragung von Projektdaten an. Q-01/Q-02 bleiben nicht bestanden.

## Zur Auswahl

**A – Striktes Ziel beibehalten; technische Isolation untersuchen (empfohlen).** Nächster begrenzter
Scope: unterstützte, durch die Anwendung durchsetzbare Windows-Isolation für Rust und WebView2
einschließlich Prozessnachfahren prüfen. Zunächst Machbarkeit und reproduzierbarer lokaler Prototyp,
kein ungeprüfter Umbau des produktiven Launchers. Abnahme mit unabhängiger ETW-Prozesszuordnung und
positiven Kontrollen, vollständigen Prozessenden, CRUD, Export/Neuimport und Neustart. Lokale
Ressourcen, Tauri-IPC, Barrierearmut, Installation und sicherer Runtime-Updatepfad dürfen nicht
stillschweigend ausfallen. Falls diese Plattform das Ziel nicht trägt, bleibt auch ein
Plattformwechsel als gesonderter größerer Entscheid offen.

**B – Fachliche Laufzeitgrenze ausdrücklich ändern.** Eine mögliche Ausnahme würde ausschließlich
konkret belegte Runtime-Konfigurations- und Komponentenmetadaten betreffen. Vor Annahme müssen
erlaubte Daten, Zielklassen, Nutzerinformation und überprüfbare Grenzen präzise definiert und
Pflichtenheft, Abnahmematrix und Datenschutzaussagen gemeinsam geändert werden. Die aktuelle Messung
beweist keine zulässigen Body-Inhalte und reicht daher allein nicht zur Freigabe dieser Ausnahme.
Keine pauschale Ausnahme für Manuskripte, Bilder, Prompt-Kontext oder beliebige Telemetrie. Das wäre
eine Produktentscheidung, kein technischer Bugfix; sie ist bisher nicht autorisiert.

## Bis zur Entscheidung

Keine neuen Abschaltflags, Firewall-/OS-Datenschutzänderungen, TLS-/Sandbox-Abschaltung oder
automatische Providerfreigabe. Unterstütztes Custom-Crash-Reporting ist ein separater, begrenzter
Mechanismus und löst die beobachteten Routen nicht. Diagnoselogs gehören nicht in Releases. Issue
#134 bleibt offen. Eine Entscheidung für A ist zunächst eine Arbeitsreihenfolge und keine Aussage,
dass eine tragfähige Isolation bereits existiert.
