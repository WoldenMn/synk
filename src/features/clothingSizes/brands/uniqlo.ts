import type { Brand } from "./types";

// Uniqlo — grilles HOMME sourcees (Task 5/6).
// Sizing Uniqlo base sur les standards asiatiques : taille « vrai » a legerement petit
// vs marques US/EU ; beaucoup d'acheteurs prennent une taille au-dessus pour un fit ample.
//
// top : tour de POITRINE corporel (cm) -> taille alpha. maxValue = borne HAUTE de la bande.
// Sources (concordantes a +-1 cm) :
//   https://sizecharter.com/brands/uniqlo/mens
//   https://qianshiwear.com/size-chat-tools/uniqlo-mens-size-chart/
//   https://www.uniqlo.com/us/en/special-feature/measurement
//
// pants : tour de TAILLE corporel (cm) -> taille alpha (Uniqlo publie aussi en pouces :
// XS 28", S 30", M 31-32", L 34", XL 38", XXL 40"). maxValue = borne HAUTE en cm.
// Sources :
//   https://www.size.ly/size-chart/uniqlo
//   https://qianshiwear.com/size-chat-tools/uniqlo-mens-size-chart/
//   https://www.sizedepo.com/sc/chart/uniqlo-men-pants/1103
export const uniqlo: Brand = {
  id: "uniqlo",
  label: "Uniqlo",
  grids: {
    top: [
      { maxValue: 89, label: "XS" },
      { maxValue: 97, label: "S" },
      { maxValue: 104, label: "M" },
      { maxValue: 112, label: "L" },
      { maxValue: 120, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    pants: [
      { maxValue: 71, label: "XS" },
      { maxValue: 76, label: "S" },
      { maxValue: 80, label: "M" },
      { maxValue: 88, label: "L" },
      { maxValue: 96, label: "XL" },
      { maxValue: 999, label: "XXL" },
    ],
    // Chemise homme (tour de COU cm -> taille de col Uniqlo).
    // Uniqlo ne vend ses chemises habillees qu'en tailles alpha (XS-XXL) mais publie
    // la mesure de COL en cm par taille (le col d'une chemise se ferme autour du cou,
    // donc col EU en cm = tour de cou). Labels = taille de col Uniqlo en cm.
    // maxValue = borne HAUTE du tour de cou (cm) pour ce col.
    // Sources (concordantes, col en cm et pouces -> cm au 1/10) :
    //   https://sizecharter.com/brands/uniqlo/mens  (col cm : XS 36, S 38, M 40, L 42, XL 44, XXL 46,5)
    //   https://www.shirtdetective.com/uniqlo-shirt-fit-guide/  (col pouces : 14,25/15,0/15,7/16,5/17,3/18,1)
    //   https://www.uniqlo.com/us/en/special-feature/measurement  (col = mesure officielle a plat, cm/pouce)
    shirt: [
      { maxValue: 36, label: "36" },
      { maxValue: 38, label: "38" },
      { maxValue: 40, label: "40" },
      { maxValue: 42, label: "42" },
      { maxValue: 44, label: "44" },
      { maxValue: 999, label: "46+" },
    ],
  },
};
