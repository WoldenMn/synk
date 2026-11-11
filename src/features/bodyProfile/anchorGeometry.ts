import landmarks from "./plate-landmarks.json";

export const PLATE_WIDTH = landmarks.viewBox.width;
export const PLATE_HEIGHT = landmarks.viewBox.height;

// Ordonnees des reperes anatomiques de la figure, en unites du viewBox.
export const PLATE_LANDMARKS = landmarks.y;

// `anchor.y` du catalogue se compte depuis les PIEDS : 0 = plante des pieds,
// 100 = sommet du crane. L'axe SVG, lui, descend depuis le haut : il faut donc
// inverser. Repere de controle : l'ancre `height` vaut 97, et le sommet du crane
// de la figure est a y = 21 sur 700.
export function anchorToPlatePoint(anchor: { x: number; y: number }) {
  return {
    cx: (anchor.x / 100) * PLATE_WIDTH,
    cy: (1 - anchor.y / 100) * PLATE_HEIGHT,
  };
}
