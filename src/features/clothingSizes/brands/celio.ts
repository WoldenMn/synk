import type { Brand } from "./types";

// Grilles homme Celio (marche FR/Europe), mesures corporelles en cm.
// top   : tour de POITRINE -> taille alpha (grille officielle "Jackets, Shirts, T-Shirts and Polos").
// pants : tour de TAILLE   -> taille FR/EU numerique (34..52).
// Bornes hautes (maxValue) = borne superieure de l'intervalle Celio ; derniere bande maxValue 999.
//
// Sources top (guide poitrine Celio, intervalles contigus 76/80/88/96/104/112/120) :
//   https://www.celio.com/fr-fr/guide-taille/guide-des-tailles-t-shirts.html
//   https://www.celio.com/fr-fr/guide-taille/guide-des-tailles-vestes.html
//   https://www.orangejeans.co/pages/celio-shirts-t-shirts-and-polos
//
// Sources pants (guide pantalons Celio, FR 34..52 -> taille 68..104 cm) :
//   https://www.celio.com/fr-fr/guide-taille/guide-des-tailles-pantalons.html
//   https://bevava.com/pages/celio-size-guide
//   https://www.looksize.com/brand-size-chart/celio
export const celio: Brand = {
  id: "celio",
  label: "Celio",
  grids: {
    top: [
      { maxValue: 80, label: "XS" },
      { maxValue: 88, label: "S" },
      { maxValue: 96, label: "M" },
      { maxValue: 104, label: "L" },
      { maxValue: 112, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    pants: [
      { maxValue: 70, label: "34" },
      { maxValue: 74, label: "36" },
      { maxValue: 78, label: "38" },
      { maxValue: 82, label: "40" },
      { maxValue: 86, label: "42" },
      { maxValue: 90, label: "44" },
      { maxValue: 94, label: "46" },
      { maxValue: 98, label: "48" },
      { maxValue: 102, label: "50" },
      { maxValue: 999, label: "52" },
    ],
    // Chemise homme (tour de cou cm -> taille de col FR). En couture FR/Celio la
    // taille de col EST le tour de cou en cm (col 39 = cou 39 cm), mapping identite.
    // Chaque bande borne le col a sa valeur cm ; derniere bande maxValue 999 = col 44.
    // Sources (Celio publie "FR 36..44 = tour de cou" sur son guide chemises ; barème FR standard) :
    //   https://www.celio.com/fr-fr/guide-taille/guide-des-tailles-chemises.html
    //   https://guidedestailles.com/taille-chemise-homme.html
    //   https://www.premieremanche.com/en/content/14-shirt-size-conversion-guide-fr-us-uk
    shirt: [
      { maxValue: 37, label: "37" },
      { maxValue: 38, label: "38" },
      { maxValue: 39, label: "39" },
      { maxValue: 40, label: "40" },
      { maxValue: 41, label: "41" },
      { maxValue: 42, label: "42" },
      { maxValue: 43, label: "43" },
      { maxValue: 999, label: "44" },
    ],
  },
};
