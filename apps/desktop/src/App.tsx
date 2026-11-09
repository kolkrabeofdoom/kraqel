const modules = [
  ["Schreiben", "Kapitel, Szenen und ein Editor ohne Ablenkungszirkus."],
  ["Story-Bibel", "Figuren, Orte, Beziehungen und Rückverweise."],
  ["Plot & Zeit", "Handlungsstränge und mehrere Zeitebenen."],
  ["Medien", "Lokale Assets, visuelle Profile und Stilvorgaben."],
] as const;

export function App() {
  return (
    <main className="shell">
      <header className="hero">
        <span className="eyebrow">Offline-first Autorenstudio</span>
        <h1>KRAQEL</h1>
        <p>
          Geschichten ordnen, ohne sie zu domestizieren. Deine Daten bleiben lokal; optionale KI
          bekommt nur zu sehen, was du ausdrücklich freigibst.
        </p>
        <div className="actions" aria-label="Projektaktionen">
          <button type="button">Neues Projekt</button>
          <button type="button" className="secondary">
            Projekt öffnen
          </button>
        </div>
      </header>

      <section aria-labelledby="module-heading">
        <div className="section-heading">
          <div>
            <span className="eyebrow">Version 0.1</span>
            <h2 id="module-heading">Ein Werkzeugkasten, kein Orakel</h2>
          </div>
          <span className="status">Fundament aktiv</span>
        </div>
        <div className="module-grid">
          {modules.map(([title, description], index) => (
            <article key={title}>
              <span className="module-number">0{index + 1}</span>
              <h3>{title}</h3>
              <p>{description}</p>
            </article>
          ))}
        </div>
      </section>

      <footer>
        <span>Keine Cloudpflicht. Keine Telemetrie ab Werk.</span>
        <span>GPL-3.0-or-later</span>
      </footer>
    </main>
  );
}
