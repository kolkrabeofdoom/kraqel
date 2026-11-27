# BP4b · Speichern und Revisionskonflikt unter Edit-Bindung

**19. November 2026 · Bestanden.** Der Dienst bestätigte Revision 1 und wies den zweiten
Schreibversuch mit CONFLICT ab. Die UI las den zuerst gespeicherten Text auf Revision 1 zurück
(Exitcodes 70/71, Starter 0). Beide Jobs endeten leer. Der Gast ist ausgeschaltet und die
Ausgangsplatte wieder zugeordnet. Der schreibgeschützte Gegencheck bestätigte null Netzadapter, null
aktivierte Integrationsdienste, entfernte Startbelege, Paket-DACL sowie die Image- und Policyhashes.
Ein eigener ETW-Vergleich dieses Binärstands steht aus.

Der Starter bindet ein frisches Paar an den Prüfzweck `A/edit/conflict-check` und prüft dessen
Endpunktrechte vor Resume. Nach HELLO/READY hält der Datendienst für alle vier authentisierten
Nachrichten dieselbe Szene A im Modus `edit`. Beide Schreibaufträge verwenden gültige
Katalogfassungen und unabhängige Kanal-Sequenznummern. Der reguläre BP4a-Speicherpfad entscheidet
anhand der erwarteten Inhaltsrevision.

| Sequenz | Auftrag                                                                    | Erwartete Antwort                       |
| ------- | -------------------------------------------------------------------------- | --------------------------------------- |
| 1       | Anfangsstand lesen                                                         | Zwei Zeilen `Rabe` und `42`, Revision 0 |
| 2       | Bei erwarteter Revision 0 die zweite Zeile auf `43` setzen                 | `{"revision":1}`                        |
| 3       | Bei weiterhin erwarteter Revision 0 die zweite Zeile auf `42` zurücksetzen | `{"error":"CONFLICT"}`                  |
| 4       | Endstand erneut lesen                                                      | Zwei Zeilen `Rabe` und `43`, Revision 1 |

Die UI kontrolliert die authentisierten Antworten und den Endtext durch Rücklesen aus dem
Win32-Feld. Ein Rechtefehler, ein verworfener Frame oder eine weitere Revision genügt dem Test
nicht. Die Prüfung umfasst den speicherresidenten synthetischen Katalog während der Lebenszeit des
Diensts. Persistenz und Wiederanlauf stehen separat aus.

**Wiederholung.** Die vier Forschungsprogramme wie bei der
[Schreibabweisung unter Lesebindung](native-win32-denied-save.md) bauen und die Hashes im frischen
Offline-Gastoverlay an die erzwungene UMCI-Richtlinie und den Paketanker binden. In der interaktiven
Gastsitzung `pipe-guest.ps1 -Trial edit-conflict` starten. Exit 70 des Diensts verlangt die reguläre
Speicherbestätigung und den Konflikt vor der abschließenden Leseantwort. Exit 71 der UI verlangt
zusätzlich den erfolgreichen Vergleich der Schlussrevision und des Feldinhalts. Den Starter-Exit,
leere Jobs und die Wiederherstellung der Gastgrenze prüfen.

## Geprüfte Binärdateien

| Gastdatei        | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| native-leaf.exe  | `804803c397a457415e194ed625a2b088251544747975a3e5985ea69215a0fd1d` |
| pipe-check.exe   | `3749bf9088e4a437cb8e29421f581468d6c5ff325205545aa80ed7bd9f232290` |
| pipe-other.exe   | `8bff7b896bc71902990c0ca5be0804784585c340c60eb728ff422a031e7e130b` |
| pipe-service.exe | `fa3115d839dd54162865fc9a00d2fa76dba27f2299bb55891671e4cee93d1c2d` |
