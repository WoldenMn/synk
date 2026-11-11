import type { Brand } from "./types";

// Grilles HOMME Pull & Bear, sourcees (3 sources concordantes ; primaire pullandbear.com
// en 403 sur WebFetch au moment du releve). Bornes = tour de POITRINE (top) et tour de
// TAILLE (pants) en cm, valeur haute de chaque bande.
export const pullbear: Brand = {
  id: "pullbear",
  label: "Pull & Bear",
  grids: {
    // Tour de poitrine (cm) -> taille alpha. Pas ~8 cm, bornes retenues = looksize
    // (confirmees par les points centraux matchnfit : XS 87 / S 94 / M 101 / L 108 / XL 115).
    // https://www.looksize.com/brand-size-chart/pull-and-bear
    // https://www.orangejeans.co/pages/pull-bear-size-guide
    // https://www.matchnfit.com/marques/pull-bear.html
    top: [
      { maxValue: 86, label: "XS" },
      { maxValue: 94, label: "S" },
      { maxValue: 102, label: "M" },
      { maxValue: 110, label: "L" },
      { maxValue: 118, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    // Tour de taille (cm) -> taille alpha. Bornes retenues = fourchettes orangejeans
    // (S 76-79 / M 81-84 / L 86-89 / XL 91-94 / XXL 97-104) ; XS borne 73 derivee du
    // point central matchnfit (70) en continuite avec S.
    // https://www.orangejeans.co/pages/pull-bear-size-guide
    // https://www.matchnfit.com/marques/pull-bear.html
    // https://www.looksize.com/brand-size-chart/pull-and-bear
    pants: [
      { maxValue: 73, label: "XS" },
      { maxValue: 79, label: "S" },
      { maxValue: 84, label: "M" },
      { maxValue: 89, label: "L" },
      { maxValue: 94, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
  },
};
