import type { Brand } from "./types";

export const zara: Brand = {
  id: "zara",
  label: "Zara",
  grids: {
    // Haut homme (tour de poitrine cm -> taille alpha Zara).
    // Zara publie le guide homme en tailles alpha (S/M/L/XL/XXL).
    // Sources : https://www.zara.com/us/en/help-center/MySize (guide officiel) ;
    //   https://www.size.ly/size-chart/zara ; https://sizecharter.com/brands/zar/mens
    top: [
      { maxValue: 93, label: "S" },
      { maxValue: 98, label: "M" },
      { maxValue: 103, label: "L" },
      { maxValue: 108, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    // Pantalon homme (tour de taille cm -> taille alpha Zara).
    // Guide officiel Zara homme en alpha. Plages source : S 71-80, M 80-88,
    // L 89-92, XL 95-101. Le trou source 92-95 est comble en etendant L jusqu'a
    // 94 (XL prend le relais a partir de 95, en catch-all pour 95 et au-dela).
    // Sources : https://www.zara.com/us/en/help-center/MySize (guide officiel) ;
    //   https://www.size.ly/size-chart/zara ; https://sizecharter.com/brands/zar/mens
    pants: [
      { maxValue: 80, label: "S" },
      { maxValue: 88, label: "M" },
      { maxValue: 94, label: "L" },
      { maxValue: 999, label: "XL" },
    ],
  },
};
