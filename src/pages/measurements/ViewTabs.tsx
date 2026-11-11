export type MeasurementsView = "sheet" | "sizes" | "body";

interface Props {
  view: MeasurementsView;
  setView: (view: MeasurementsView) => void;
}

export function ViewTabs({ view, setView }: Props) {
  return (
    <div className="inline-flex overflow-hidden rounded-lg border">
      <button
        type="button"
        aria-pressed={view === "sheet"}
        onClick={() => setView("sheet")}
        className={`px-4 py-1.5 text-sm font-medium ${
          view === "sheet" ? "bg-primary text-primary-foreground" : "text-muted-foreground"
        }`}
      >
        Fiche
      </button>
      <button
        type="button"
        aria-pressed={view === "sizes"}
        onClick={() => setView("sizes")}
        className={`px-4 py-1.5 text-sm font-medium ${
          view === "sizes" ? "bg-primary text-primary-foreground" : "text-muted-foreground"
        }`}
      >
        Tailles
      </button>
      <button
        type="button"
        aria-pressed={view === "body"}
        onClick={() => setView("body")}
        className={`px-4 py-1.5 text-sm font-medium ${
          view === "body" ? "bg-primary text-primary-foreground" : "text-muted-foreground"
        }`}
      >
        Tours de corps
      </button>
    </div>
  );
}
