import type { Brand } from "./types";

// Grilles homme Jack & Jones — sourcees (guide officiel + 2 sources independantes).
// maxValue = borne haute (cm) de la bande : mesure <= maxValue -> label.
//
// top : tour de POITRINE (cm) -> taille alpha. Slim fit europeen (le guide conseille
// de prendre au-dessus pour une coupe plus ample). Points de mensuration corps
// XS 92 / S 96 / M 100 / L 104 / XL 112 / XXL 120 ; bornes hautes = mi-chemin vers la taille suivante.
// Sources :
//   https://www.jackjones.us/en-us/content/size-guide         (officiel, primaire)
//   https://sizgu.com/size-charts-jack-jones/
//   https://www.looksize.com/brand-size-chart/jack-and-jones
//
// pants : tour de TAILLE (cm) -> label W (specialite jean de la marque).
// Points de mensuration corps W28 76 / W29 78.5 / W30 81 / W31 83.5 / W32 86 /
// W33 88.5 / W34 91 / W36 96 / W38 101 ; bornes hautes = mi-chemin vers la taille suivante.
// Sources :
//   https://www.jackjones.us/en-us/content/size-guide         (officiel, primaire)
//   https://sizgu.com/size-charts-jack-jones/
//   https://www.looksize.com/brand-size-chart/jack-and-jones
export const jackjones: Brand = {
  id: "jackjones",
  label: "Jack & Jones",
  grids: {
    top: [
      { maxValue: 94, label: "XS" },
      { maxValue: 98, label: "S" },
      { maxValue: 102, label: "M" },
      { maxValue: 108, label: "L" },
      { maxValue: 116, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    pants: [
      { maxValue: 77.25, label: "W28" },
      { maxValue: 79.75, label: "W29" },
      { maxValue: 82.25, label: "W30" },
      { maxValue: 84.75, label: "W31" },
      { maxValue: 87.25, label: "W32" },
      { maxValue: 89.75, label: "W33" },
      { maxValue: 93.5, label: "W34" },
      { maxValue: 98.5, label: "W36" },
      { maxValue: 999, label: "W38" },
    ],
  },
};
