# ADR 0035 – Unabhängige Prozess- und Imagefreigabe für BP3

**Entschieden am 18. November 2026.** Der Fixed-Version-WebView2-Pfad bleibt an der C4-Startsperre.
Die geprüften Mechanismen liefern auf dem normalen Windows-Desktop keine belegte Kombination aus
früh wirksamer Imagefreigabe, geschlossener Starterkette und Zuweisung effektiver Rechte für jede
WebView-Rolle. Diese Entscheidung bewertet die dokumentierten Schnittstellen und den aktuellen
KRAQEL-Pfad. Andere Architekturen bleiben prüfbar.

Der native Gegenversuch am Stand `b8a33aa` zeigte eine synthetische Kindgeneration, die der äußere
Debugger bei eigener Debugkette nicht sah. Job und AppContainer blieben erhalten. Der C4-Controller,
Worker und Runtime-Einstieg verweigern die Ausführung, solange die präventive Grenze fehlt.
[Debug-Gegenprobe](../../testing/product-start-p2c-bp3-debug-guardian.md),
[Startsperre](../../testing/product-start-p2c-bp3-admission.md),
[Präloadvertrag](../../data-format/product-start-p2c-c4-preload-v1-draft.md).

## Bewertung

| Mechanismus                                                       | Durchsetzung und Einsatzgrenze                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CreateProcess` mit Job, Token, Handleliste und Child-Restriction | Geeignet für eigene suspendierte Starts und belegte Blattrollen. Eine Anwendung kann so Starts durch ihren eigenen Broker kontrollieren. Der Nachweis umfasst die effektiven Prozess- und Handle-Rechte, weil die dokumentierte Child-Restriction allein keine vollständige Grenze bildet. Jobzugehörigkeit begrenzt Ressourcen und Lebensdauer, sie beschreibt keine Rolle. [Windows-Creation-Attribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)                                                                                                                                                                                                                                                                                                           |
| App Control for Business mit erzwungener UMCI                     | Betriebssystemseitige Imagefreigabe auch für unsignierte Dateien über Hashregeln. Die Richtlinie gilt geräteweit. Ein Authenticode-/PE-Hash deckt andere Bytes ab als der vollständige SHA-256 des KRAQEL-Inventars. Beide Prüfungen sind für unterschiedliche Zwecke erforderlich. Dieselbe freigegebene EXE kann mehrere Rollen ausführen. Die Hashregel entscheidet weder über ihre Rolle noch über deren Rechte. [Regeln und Hashs](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/select-types-of-rules-to-create), [Geltungsbereich](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/feature-availability)                                                                        |
| AppLocker mit erzwungenen EXE- und DLL-Regelsammlungen            | Dokumentierte Datei- und Identitätsregeln für Konten und Gruppen. Je nach effektiver Richtlinie sind EXE und DLL separat zu prüfen. Diese Identitäten binden keine einzelne Prozessgeneration an ihren Erzeuger, ihre Handles und ihre WebView-Rolle. [Regelsammlungen](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/applocker/working-with-applocker-rules)                                                                                                                                                                                                                                                                                                                                                                                                      |
| Kernel-Prozesscallback und Imagebenachrichtigung                  | Der Prozesscallback kann eine Erzeugung verweigern. Die Imagebenachrichtigung erfolgt nach dem Mapping und hat keinen dokumentierten Rückgabewert zur Freigabeentscheidung. Die dokumentierte Abschnittssynchronisation eines Minifilters trägt keine allgemeine rollenbezogene Ausführungspolicy. Ein eigener Treiber würde eine neue vertrauenswürdige Komponente mit eigener Auslieferung und Abnahme erfordern. [Prozesscallback](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/ns-ntddk-_ps_create_notify_info), [Imagebenachrichtigung](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddk/nc-ntddk-pload_image_notify_routine), [Abschnittssynchronisation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ifs/flt-parameters-for-irp-mj-acquire-for-section-synchronization) |
| WebView2-Prozessinformationen und Debugereignisse                 | WebView2 liefert Beobachtungen über bereits gestartete Prozesse und lässt Crashpad in dieser Prozessliste aus. Die native Gegenprobe widerlegt die Vollständigkeit der äußeren Debugkette. Beide Quellen bleiben für Diagnose und Gegenprüfung geeignet. [WebView2-Prozessinformationen](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment8), [Debugvererbung](https://learn.microsoft.com/en-us/windows/win32/debug/process-functions-for-debugging)                                                                                                                                                                                                                                                                                                                                    |
| Offline-Gast mit eigener Richtlinie                               | Eine separat eingerichtete VM kann die Richtlinie vom Arbeitsrechner trennen und Netzwerk sowie Hostfreigaben an der Gastgrenze begrenzen. Diese Grenze sagt nichts über die Rechte einzelner Rollen im Gast aus. Microsoft dokumentiert für Windows Sandbox standardmäßig aktives Netzwerk und Zwischenablage. Gastkonfiguration und tatsächliche Kanäle sind Teil eines eigenen Nachweises. [Windows Sandbox](https://learn.microsoft.com/en-us/windows/security/application-security/application-isolation/windows-sandbox/windows-sandbox-configure-using-wsb-file)                                                                                                                                                                                                                                                                   |

## Entscheidung und nächster Nachweis

Als **Referenz für eine getrennte Machbarkeitsprobe** wählen wir erzwungene UMCI im entbehrlichen
Offline-Gast, kombiniert mit einem einzigen vertrauenswürdigen Erzeuger für eigene synthetische
Rollen. Die Betriebssystemrichtlinie kontrolliert die Ausführung der Images. Der Erzeuger bindet die
Rolle samt Rechten vor der ersten Ausführung. Die Referenz ist ein Prüfkandidat für eigene
Rollenprozesse. Für den aktuellen WebView2-Kandidaten fehlt ein unterstützter Weg, **alle** intern
erzeugten Rollen einschließlich Crashpad und späterer Rendererstarts durch diesen Erzeuger zu
führen. Eine geräteweite Richtlinie auf dem Arbeitsrechner ist kein Bestandteil der Entscheidung.

Für einen ersten begrenzten Versuch gelten diese Abnahmekriterien:

1. Der Offline-Gast und seine effektive erzwungene Richtlinie sind vor dem Starter aktiv. Der
   Startanker bindet Gaststand, Richtlinie, Starter und Plattformbasis. Ein fehlender oder
   abweichender Anker sperrt. Die Prüfung nutzt ausschließlich synthetische Artefakte.
   Netzwerkadapter, Dateifreigaben, Zwischenablage und sonstige Hostkanäle werden an der Gastgrenze
   geprüft.
2. Ein inventarisiertes unsigniertes Testimage läuft unter der vorgesehenen Rolle. Je ein fremdes
   EXE- und DLL-Image sowie eine spätere DLL-Nachladung werden durch die Betriebssystemrichtlinie
   vor Ausführung abgewiesen. Der erlaubte Kontrollmarker muss funktionieren. Der fehlende
   Ablehnungsmarker allein genügt nicht. Maßgeblich sind der native Rückgabestatus, die effektive
   Richtlinie und das beobachtete Testergebnis.
3. Zwei Rollen mit derselben zugelassenen EXE erhalten nachprüfbar unterschiedliche effektive
   Rechte. Nur der Erzeuger startet sie. Direkte Kindstarts scheitern auch bei beantragter eigener
   Debugkette. Token, Handleliste und Job werden vor Resume an der gehaltenen Prozessgeneration
   geprüft. Meldungen und Kommandozeilen dienen höchstens als Diagnose.
4. Ausführbarer dynamischer Speicher braucht eine eigene Regel. Für eine Rolle ohne JIT wird
   `ProhibitDynamicCode` ohne Opt-out oder Downgrade als separater Kandidat geprüft. Für Rollen mit
   JIT bleibt eine kompatible, wirksam begrenzte Regel offen. Ein Imagehash deckt JIT-Code nicht ab.
   [Windows Dynamic Code Policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy).
5. Fällt der Erzeuger aus, bleibt die Gruppe gesperrt, bis ihr Ende belegt ist. Ein gezielter
   Ausfallfall und ein Neustart mit fehlender oder geänderter Richtlinie prüfen diese Grenze. Neue
   Binärstände brauchen ihren eigenen Netzbeleg vor einer Freigabe. Die Kanäle außerhalb des
   Prozessbaums gehören zur Datenschutzprüfung des Gasts.

Die Bewertung wurde anhand der Primärdokumentation und des bestehenden Quell- und Prüfstands
abgeschlossen. Die vorhandene synthetische Gastprobe hat zwei Blattrollen und die Sperre fremder
EXE-/DLL-Images unter erzwungener UMCI nachgewiesen. Die fremde statische Start-DLL wurde im Gast
durch UMCI gesperrt, der Positivimport lief. Die effektiven Rechte am synthetischen Markerobjekt
sind vor Resume geprüft. Der eigene Netzbeleg für diesen Binärstand bleibt offen. Bei fehlender
Prüfrichtlinie und geänderter Richtlinienversion derselben ID ist die Wiederanlaufsperre des
synthetischen Einstiegs belegt. Der gezielte Starterausfall mit laufendem synthetischem
Blattprozess, dessen Ende und der Leerstand der Gruppe sind im Offline-Gast bestätigt. Für die
tatsächlichen WebView-Rollen fehlt der präventive Erzeugungsweg.
[Ergebnis und Beleggrenzen](../../testing/product-start-p2c-bp3-umci-guest.md). BP3 insgesamt und
`RuntimeStartAllowed` bleiben gesperrt. `DataAllowed` benötigt zusätzlich die getrennte BP4-Abnahme.
Bis dahin erhält die Referenz weder echte Projektinhalte noch dauerhafte Kontrollschlüssel. Die
normale KRAQEL-Auslieferung bleibt unsigniert und das strikte Datenschutzziel gilt.
