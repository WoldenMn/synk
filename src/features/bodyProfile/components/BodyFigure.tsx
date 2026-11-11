import frontUrl from "@/assets/body/front.png";
import backUrl from "@/assets/body/back.png";
import { PLATE_HEIGHT, PLATE_WIDTH, anchorToPlatePoint } from "../anchorGeometry";
import type { BodyMeasurement } from "../types";

interface BodyFigureProps {
  view: "front" | "back";
  highlight?: string[];
  annotations?: BodyMeasurement[];
  className?: string;
}

export function BodyFigure({ view, highlight = [], annotations = [], className }: BodyFigureProps) {
  const src = view === "front" ? frontUrl : backUrl;
  const visible = annotations.flatMap((m) =>
    m.anchor?.view === view ? [{ ...m, anchor: m.anchor }] : []
  );

  return (
    // `self-start` : sans lui, le conteneur est etire a la hauteur de la cellule
    // de grille voisine et le calque, en `inset-0`, s'etire avec lui — les
    // pastilles finissent bien en dessous de la silhouette.
    <div className={`relative self-start ${className ?? ""}`}>
      <img
        src={src}
        alt={view === "front" ? "Silhouette de face" : "Silhouette de dos"}
        className="block w-full drop-shadow-[0_0_1px_hsl(var(--foreground)/0.5)] dark:drop-shadow-none"
      />
      <svg
        viewBox={`0 0 ${PLATE_WIDTH} ${PLATE_HEIGHT}`}
        className="absolute inset-0 h-full w-full"
        aria-hidden="true"
      >
        {visible.map((m) => {
          const { cx, cy } = anchorToPlatePoint(m.anchor);
          const isOn = highlight.includes(m.slug);
          return (
            <circle
              key={m.slug}
              cx={cx}
              cy={cy}
              r={isOn ? 7 : 4}
              className={isOn ? "fill-primary" : "fill-muted-foreground/50"}
            />
          );
        })}
      </svg>
    </div>
  );
}
