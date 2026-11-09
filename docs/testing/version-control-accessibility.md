# Manueller Testplan: Versionsansicht

Automatisierte Tests decken Snapshotdeterminismus, Manipulationserkennung, Pfadfreigabe,
Git-Status/Diff/Commit und den TypeScript-Commandvertrag ab. Folgende Bedienfälle bleiben manuell:

1. Projekt öffnen und ausschließlich mit der Tastatur zum Bereich **Versionen** wechseln.
2. **Snapshot erzeugen** auslösen; Statusmeldung, Dateianzahl und geänderte Dateien müssen ohne
   Fokusverlust verständlich sein.
3. **Lesend prüfen** auslösen; Erfolg oder bereinigter Fehler muss über den Live-Bereich hörbar
   sein.
4. Den Unified Diff per Tastatur fokussieren und horizontal sowie vertikal lesen.
5. Eine Commitnachricht eingeben und committen; der verkürzte Commit muss sichtbar werden, der
   Zustand anschließend „unverändert“ lauten.
6. Mit Windows-Sprachausgabe oder NVDA prüfen, dass Navigation, Schaltflächenzustände,
   Formularlabel, Status und Fehler in sinnvoller Reihenfolge ausgegeben werden.

Keine Testperson soll dafür echte Manuskripte verwenden. Selbst Testkrähen verdienen
Datensparsamkeit.
