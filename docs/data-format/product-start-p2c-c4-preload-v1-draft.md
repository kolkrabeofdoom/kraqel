# P2c C4b: präventive Modul- und Rollenbindung

Status: **Vertragsentwurf v1, 18. November 2026; nicht implementiert und keine Startautorität.**
Ergänzt [C4](product-start-p2c-c4-v1-draft.md),
[ADR 0034](../architecture/decisions/0034-webview-network-boundary-proposal.md) und den
[gemessenen Sperrstand](../testing/product-start-p2c-c4b-module-gate-etw.md). MUSS bezeichnet eine
neue Implementierungsanforderung. Native UI-ABI bleibt 0; unverändert unsignierte KRAQEL-Programme
zulässig, keine Zertifikats-/Signaturpflicht für das Produkt.

## 1. Sicherheitsziel und verifizierte Grenze

Nicht zugelassener ausführbarer Code MUSS vor seiner ersten Ausführung gesperrt sein. Das gilt für
den Wächter, den Root einschließlich initialer DLLs, alle Runtimegenerationen und spätere
Nachladungen. Rollenbezogene Rechte und Kindstartregeln MÜSSEN vor dem ersten Ausführen der
betreffenden Generation wirksam sein. Keine nachträgliche Identifizierung darf rückwirkend Rechte
erteilen. Die vertraute Basis bleibt Windows einschließlich seiner privilegierten Durchsetzung; dies
ist kein Schutzversprechen gegen einen kompromittierten Kernel oder privilegierte lokale Kontrolle.
Installierte Schutzsoftware wird dadurch nicht automatisch zum erlaubten Paketmodul.

Die Start-Vertrauenskette des Wächters einschließlich seines minimalen Starters MUSS ausdrücklich
belegt sein, bevor er produktive dauerhafte Schlüssel oder Projektdaten erhält. Ein Hash seiner EXE
beweist nicht seine geladenen DLLs; eine eigene nachträgliche Erfolgsmeldung belegt nicht seinen
früheren Start. Diese Host-Startbindung ist noch kein vorhandener nativer Nachweis.
Root-Mitigationen allein dürfen diese offene Voraussetzung nicht verdecken; daraus wird keine
Ausnahme für fremden Code im Wächter.

Der Ist-Stand prüft Paketbytes und Dateiidentitäten, setzt AppContainer/Job/Handleliste beim
Rootstart und stoppt einen authentisierten Root bei fremden Modulaufnahmen vor Continue. Er setzt
**keine** eigenen Creation-Mitigation-/Child-Process-Policy-Attribute und liest deren wirksame
Einstellungen nicht zurück. Der Root läuft bereits vor seiner Modulaufnahme.
[Attribute im Code](../../scripts/runtime-isolation/probe/src/windows.rs),
[Startfolge](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/controller.rs).
Die vorhandene Modulsperre und ETW erfüllen deshalb dieses präventive Ziel noch nicht.

DLL-Einstiegscode kann beim Laden ausgeführt werden. Daraus folgt für diesen Vertrag: Eine erst im
Programmeinstieg installierte Policy oder spätere Modulaufnahme ist für initiale DLLs zu spät.
[Microsoft: DllMain](https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain).

## 2. Drei getrennte Nachweise

| Nachweis                | MUSS belegen                                                                                                        | Genügt allein nicht                                                                            |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Paketidentität          | Exaktes inventarisiertes Image/Modul, Hash und gehaltene native Identität; erlaubte Windows-Plattformgrenze         | Gleicher Dateiname, Pfadpräfix, Hersteller oder gültige Signatur                               |
| Präventive Durchsetzung | Unterstützter, vor Ausführung wirksamer Mechanismus für jedes erforderliche Lade-/Startziel; native wirksame Policy | Angefragte Flags, Auditmodus, Konfigurationsdigest oder späterer Snapshot                      |
| Rollenautorität         | Generation, zugelassener Erzeugungsweg, endliche Rolle, effektives Token/ACL/Job und nicht delegierte Handles       | API-Kind, Kommandozeile, gleiche SID, Eltern-PID oder ein MAC ohne nativen Kanal-/Prozessbezug |

Alle drei Nachweise sind zusätzlich erforderlich. Ein Mechanismus darf nur die Eigenschaft erfüllen,
die er tatsächlich durchsetzt. Unbekannt, nicht unterstützt oder nicht unabhängig prüfbar bedeutet
gesperrt. Auch eine vollständig unterstützte Sammlung grober Mitigationsflags ist keine automatische
Paket- oder Rollenattestierung.

## 3. Windows-Mechanismen und ihre Grenzen

| Mechanismus                                   | Dokumentierte Wirkung                                               | Vertragsentscheidung                                                                                      |
| --------------------------------------------- | ------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Creation-Mitigation-Attribut                  | Policy wird mit der Prozesserzeugung angegeben                      | Erforderliche frühe Regeln dort setzen; kein späteres Reparieren fehlender Regeln                         |
| `GetProcessMitigationPolicy`                  | Wirksame Einstellungen am Prozesshandle abfragen                    | Eigene gehaltene native Generation prüfen; Lesefehler/Abweichung sperren                                  |
| `NoRemoteImages`, `NoLowMandatoryLabelImages` | Bilder von entfernten Geräten bzw. mit Low-Mandatory-Label abweisen | Als frühe Schutzregeln prüfen; übrige lokale Module sind dadurch nicht paketgebunden                      |
| `PreferSystem32Images`/DLL-Suchpfade          | Suchreihenfolge beeinflussen                                        | Keine Positivliste und kein Ersatz für präventive Paketbindung                                            |
| Binary-Signature-Policy                       | Zulassung nach Microsoft-/Store-/WHQL-Signaturklassen               | Höchstens gesondert geprüfte Härtung; weder exakter Pakethash noch Ausnahme für den beobachteten Decoder  |
| Child-Process-Policy                          | Kindprozesserzeugung einschränken                                   | Für einen synthetischen Blattprozess prüfen; kein WebView-Rollenfilter                                    |
| Dynamic-Code-Policy                           | Erzeugung/Änderung ausführbaren Codes beschränken                   | Eigenen rollenbezogenen JIT-/Codevertrag verlangen; keine ungeprüfte Abschaltung von Runtime-Schutzregeln |

Primärquellen:
[Creation-/Child-Attribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
[Rücklesen](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocessmitigationpolicy),
[Image-Load-Policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_image_load_policy),
[DLL-Suche](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order),
[Signaturpolicy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_binary_signature_policy),
[Dynamic-Code-Policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy).
Die Sperrentscheidungen sind KRAQEL-Anforderungen, keine zusätzlichen Garantien dieser APIs.

Unterstützte Bits MÜSSEN für den konkreten Windows-/Architekturstand ermittelt werden. Ein
unbekanntes oder benötigtes nicht unterstütztes Bit darf nicht durch Maskieren verschwinden.
Widersprüchliche On-/Off-Einstellungen, Audit statt erforderlicher Durchsetzung und ungeprüfte
Defaultwerte sperren. Kindprozesse erben keine vertragliche Freigabe allein aus der Root-Abfrage.
Bei verlangter Dynamic-Code-Sperre sind Thread-Opt-out und Remote-Downgrade unzulässig; eine
notwendige JIT-Kompatibilität ist ein offener Rollenvertrag und kein stiller Fallback.

Nach Microsoft kann die Child-Restriction bei Zugriff auf entsprechend privilegierte fremde
Prozesshandles umgangen werden. Deshalb gehören Handleabschluss und effektive Rechte zum Nachweis;
„Kindstart verboten“ ist kein isoliert ausreichendes Sicherheitsargument. Eine solche
Blattprozessregel darf nicht abgeschaltet werden, um WebView anschließend doch zu starten. Dafür ist
ein getrennt belegter Mehrprozess-Erzeugungsweg erforderlich.

## 4. Vorgesehene Typen und Bindung

Die folgenden Namen sind **Entwurfsnamen**, keine vorhandenen Rust-Typen, Decoder oder Commands.

| Typ                 | Erforderlicher Inhalt                                                                                                                                                           |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PreloadPolicyV1`   | Version/Zweck, Windows-x64, Paket-/Controller-/Plattformpolicy-Digests, geschlossene Rollenliste und endliche Gesamtgrenze                                                      |
| `PreloadRoleRuleV1` | C4-Rolle, inventarisiertes Image, Instanzgrenze, Argument-/Token-/Handlepolicy, benötigte frühe Mitigationen, Kindstartregel, erforderlicher Lade-Durchsetzungsmechanismus      |
| `PreloadSupportV1`  | Gebundener OS-/Architekturstand, Wächter-Startanker, verlangte und unterstützte Eigenschaften, unbekannte/unerfüllte Anforderungen; keine Freigabe aus bloßem Versionsvergleich |
| `PreloadWitnessV1`  | Launch, Nonce, Paket-/Policydigest, Wächter- und Prozessgeneration, kontinuierliche Jobinstanz, tatsächlich gelesene Policy und belegter Erzeugungs-/Lademechanismus            |
| `PreloadDecisionV1` | Nur `SyntheticLeafEligible` oder stabile Ablehnung; keine Runtime-/Datenautorität in diesem Teilmodell                                                                          |

Pflichtfelder, Version, Zweck und Rollen sind geschlossen; unbekannte Werte und Null-Digests
sperren. Der nächste reine Modellschritt MUSS deterministische, domänenseparierte, begrenzte
Codierung samt Golden-Bytes festlegen. Höchstens 16 KiB je Policydeskriptor, maximal sechs
C4-Rollen, keine Duplikate; Root genau einmal mit Instanzgrenze 1. Alle Instanzgrenzen positiv, ihre
Summe höchstens Gesamtgrenze, Gesamtgrenze höchstens 16 für diese Forschungsstufe. Imageverweise
nach den vorhandenen portablen C4-Pfadregeln, höchstens 1.024 UTF-8-Bytes. Keine freien
Windows-Bitmasken, DLL-Pfade oder Herstellerlisten aus einer UI. Angeforderte Schutzregeln sind
semantisch typisiert; ihre native Übersetzung ist gesondert zu prüfen.

Reservierte reine Modellzwecke: `KRAQEL-C4-PRELOAD-POLICY\0\x01` für die künftige kanonische Policy
und `KRAQEL-C4-PRELOAD-MODEL\0\x01` für die künftige Modellsimulation. Noch keine implementierten
Decoder oder nativen MAC-/Grantdomänen; keine bestehenden Domains übernehmen. Die vollständige
Bytecodierung wird in BP1 mit dem Code und verbindlichen Golden-Bytes ergänzt.

Modellwitnesses dürfen native Belege nur simulieren und niemals einen nativen Grant erzeugen.
Spätere echte Witnesses entstehen ausschließlich intern beim Wächter aus gehaltenen Handles; kein
Import aus JSON, Diagnosen oder einem selbst behaupteten Erfolgsboolean. C4-Rollen und
WebView-API-Kinds sind getrennte Namensräume, keine numerische Umwandlung zwischen ihnen.

`UiPolicyV1`/`UiRuntimePackageV1` besitzen diese zusätzlichen Bindungsfelder nicht. Sie dürfen nicht
umgedeutet werden. Eine spätere Integration verlangt einen expliziten versionierten Nachfolgevertrag
für Paket, Bindung, Bootstrap und native Records sowie alte Leserabwehr. Keine Änderung an Schema 6,
bestehenden MAC-Domänen, Marker-ABI 0 oder laufenden Stores durch diesen Entwurf. Kein neues Recht
aus vorhandenen Placeholder-Digests oder Auditdateien.

## 5. Reihenfolge und getrennte Freigaben

Die folgende Reihenfolge gilt für eine spätere C4-Runtime-/Datenzulassung. Ein gesonderter
BP2-Fähigkeitsversuch ist unten ausdrücklich als nicht freigebender Forschungszweck abgegrenzt.

1. Nach belegter Wächter-Startkette unter derselben exklusiven Kontrollsperre Paket, Profil,
   Datenbindung, Lease, Budget und native Mechanismen prüfen. Bereits unerfüllbare Anforderungen
   sperren **vor CreateProcess**.
2. Launch einmalig dauerhaft verbrauchen, erforderliche Records reservieren; Root suspendiert mit
   atomarer Jobzuweisung, Token-/Handlebeschränkungen und allen erforderlichen frühen
   Creation-Policies anlegen. Erzeugungsfehler darf keine erfolgreiche Bindung vortäuschen.
3. Image, Generation, Job, Token und wirksame Policies am gehaltenen Root zurückprüfen. Ohne belegte
   präventive Durchsetzung kein Resume. Eine spätere Root-MAC kann das nicht ersetzen.
4. Rootbootstrap vollständig authentisieren. Kurzlebigen Schlüssel/Bootstraphandles vor Runtime
   schließen/löschen; permanente Kontrollautorität bleibt beim Wächter. Accepted ist keine Laufzeit-
   oder Datenfreigabe.
5. Ein späterer eigener `RuntimeStartAllowed`-Nachweis darf ausschließlich eine leere Runtime
   zulassen, wenn auch sämtliche möglichen dynamischen Kind-/Ladewege vorher wirksam begrenzt sind.
   Nachträgliches Polling, ProcessInfosChanged oder ETW erzeugt diesen Nachweis nicht.
6. Ein getrenntes späteres `DataAllowed`-Gate verlangt zusätzlich effektive Daten-ACLs,
   Rollen-/Commandbindung und laufend wirksame Durchsetzung für spätere Kinder/Nachladungen. Bis
   dahin keine Projektdaten, DB-Handles, Projektpfade oder Texte in Root/Runtime/IPC.

`RuntimeStartAllowed` und `DataAllowed` sind hier logische Anforderungen, keine neuen persistenten
Stages oder Nachrichten. Vorhandenes Running/Continue des leeren Forschungsprotokolls wird nicht zur
produktiven Autorität. Eine einmalige vollständige Rollenaufnahme schließt zukünftige Starts nicht
ab. Rollenrechte dürfen weder nachträglich angehoben noch über geteilte Tokens/Handles unterlaufen
werden.

Crashpad ist ausdrücklich mit endlicher Rolle und erforderlichen effektiven Rechten zu behandeln.
Die
[WebView-API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment8)
spart es aus. Die beobachtete Kommandozeile `--type=crashpad-handler` bleibt Diagnose; ein fehlender
unterstützter präventiver Erzeugungs-/Rollenbeleg sperrt den Mehrprozesspfad.

## 6. Abbruch- und Kompatibilitätsvertrag

| Situation                                                                    | Zwingendes Ergebnis                                                                      |
| ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Mechanismus unbekannt/nicht unterstützt oder erforderliches Flag fehlt       | `C4_PRELOAD_UNSUPPORTED`, kein Resume/kein Runtime- oder Datenstart                      |
| Native Policy nicht lesbar, nur Audit oder abweichend                        | `C4_PRELOAD_POLICY_MISMATCH`, eigene Gruppe kontrolliert beenden                         |
| Exakte Modulbindung nur durch Signatur oder Snapshot ersetzt                 | `C4_PRELOAD_LOAD_UNPROVEN`, keine Freigabe                                               |
| Rolle/Kindweg nicht präventiv gebunden                                       | `C4_PRELOAD_ROLE_UNPROVEN`, keine Freigabe; spätere Diagnose behebt das nicht            |
| Image-/Paket-/Launch-/Generation-/Witnesskontext falsch oder wiederverwendet | `C4_PRELOAD_CONTEXT_MISMATCH`, unveränderte Daten/Bindung, keine Ersatzautorität         |
| Prozessende/Drain unklar oder Wächter verloren                               | Weiter pending; kein neues Jobobjekt, PID-Replay oder Profilverschwinden als Ersatzbeleg |

Fehlernamen sind geplante bereinigte Codes, noch keine registrierten Produktfehler. Eine
Vorprüfungsabweisung vor Launchreservierung verändert den Kontrollhead nicht; ab Reservierung bleibt
der Auftrag verbraucht, auch bei fehlgeschlagener Ressourcenanlage. Drained/Finish erst bei
bestätigtem Ende derselben kontinuierlich gehaltenen Gruppe. Vorhandene Sperrpfade bleiben bestehen;
keine automatische Migration, Reparatur oder Recovery.

Die zwei zusätzlichen Modulklassen bleiben unzulässig: Verhaltensüberwachung ist keine Ausnahme; ein
Microsoft-signierter HEVC-Decoder außerhalb des Pakets ist nicht durch seine Signatur legitimiert.
Unlesbare Pfade bleiben gesperrt. Kein Kopieren solcher Dateien aus Diagnosepfaden ins Inventar,
keine Schutzsoftwareausnahme und keine systemweite Policy-/WDAC-/Registryänderung. Benötigt ein
Kandidat diese Lockerung, ist er inkompatibel und bleibt gesperrt. Dies behauptet keine generelle
Unmöglichkeit anderer Windows-Architekturen, sondern eine offene Nachweislücke des konkreten
Kandidaten.

## 7. Begrenzte Umsetzung und Abnahme

| Teilstück                | Lieferumfang                                                                                              | Mittleres Prüfniveau                                                                                                                                              |
| ------------------------ | --------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| BP1 — als Nächstes       | Nur reine Typen, deterministische Codierung, Support-/Entscheidungsmodell; keine I/O- oder Startautorität | Rundlauf/Golden, Version/Unbekanntwerte, Grenzen, Kontextwechsel, fehlende Bits, nur Audit, fehlender Modul-/Rollenbeleg und Replay ablehnen                      |
| BP2 — danach             | Native Capability-/Creation-/Readback-Prüfung mit neuem synthetischem Blattprozess, keine Runtime/DB      | Ein regulärer begrenzter Lauf, Unsupported/Policyabweichung, ein verbotener Kindstart; Keys/Handles/Drain prüfen; gezieltes ETW der tatsächlich gestarteten Bytes |
| BP3 — weiterhin gesperrt | Unterstützten präventiven Lade- und WebView-Mehrprozessweg nachweisen                                     | Ohne diesen Nachweis kein RuntimeStartAllowed; genaue Rolle/Policy und Vererbung pro Generation, keine Umgehung durch Renderer/Crashpad                           |
| BP4 — weiterhin gesperrt | Daten-/Commandfreigabe erst nach separater effektiver ACL-/Rollenabnahme                                  | Keine reale Projektübernahme; danach erst C4c/d erwägen                                                                                                           |

Ein erfolgreicher BP2-Blattprozess beweist BP3 nicht. Signaturprüfungen, ETW und negative Tests
ergänzen den Nachweis; allein beweisen sie keine vollständige präventive Positivliste. Neue native
Ergebnisse brauchen ihren eigenen Byte-/Quellbezug. Dieser Vertrag erfordert weder Build noch
erneutes ETW; Quellen-, Verweis-, Widerspruchs- und Formatprüfung genügen. Keine historische
Prüfzahl als neuen Testlauf ausgeben. C4b insgesamt, P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen.

BP2 ist ein eigener begrenzter Forschungszweck für tatsächlich verfügbare Mitigationsfähigkeiten:
neue synthetische Fixture, ein Blattprozess, keine Runtime/DB, keine Projektpfade/-texte und keine
permanenten Kontrollschlüssel im Kind. Nicht bewiesene exakte Lade-/Rollenbindung bleibt im Ergebnis
ausdrücklich unerfüllt. `SyntheticLeafEligible` erlaubt nur die weitere native Prüfung dieses
Forschungsprofils, niemals einen realen Start aus dem reinen Modell. Die dort verlangten frühen
Regeln müssen beim Blattstart ebenfalls wirksam sein; fehlende Bits sperren auch diesen Versuch.
Kein Fallback von abgewiesener Runtimezulassung in einen solchen Probeaufruf und keine Umdeutung
eines positiven Fähigkeitsversuchs in `RuntimeStartAllowed`/`DataAllowed`.

Nur neue fixtureeigene synthetische Prüfschlüssel dürfen den BP2-Forschungswächter erreichen; keine
vorhandene produktive Kontrollablage und keine bestehende echte Schlüsselautorität.
