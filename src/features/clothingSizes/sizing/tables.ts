// Bandes triées par maxValue croissant : on choisit la 1ère bande où value <= maxValue.
export interface SizeBand {
  maxValue: number;
  eu: string;
  us?: string;
  uk?: string;
}

// Veste / costume — EN 13402 : EU = tour de poitrine (cm) / 2 ; US/UK = EU - 10.
// Centres : 48=96cm, 50=100cm (palier tous les 4 cm).
// Sources : EN 13402-3 ; hockerty.com/blog/suit-size-chart ;
// spierandmackay.com/fit-guide/suit-size-chart.
export const JACKET_BANDS: SizeBand[] = [
  { maxValue: 90, eu: "44", us: "34", uk: "34" },
  { maxValue: 94, eu: "46", us: "36", uk: "36" },
  { maxValue: 98, eu: "48", us: "38", uk: "38" },
  { maxValue: 102, eu: "50", us: "40", uk: "40" },
  { maxValue: 106, eu: "52", us: "42", uk: "42" },
  { maxValue: 110, eu: "54", us: "44", uk: "44" },
  { maxValue: 114, eu: "56", us: "46", uk: "46" },
  { maxValue: 999, eu: "58", us: "48", uk: "48" },
];

// Chemise — col EU = tour de cou (cm) ; US/UK en pouces.
// Source : nordstrom.com size guides (EU 39 ≈ 15.5", EU 41 ≈ 16").
export const SHIRT_BANDS: SizeBand[] = [
  { maxValue: 37.5, eu: "37", us: "14.5", uk: "14.5" },
  { maxValue: 38.5, eu: "38", us: "15", uk: "15" },
  { maxValue: 39.5, eu: "39", us: "15.5", uk: "15.5" },
  { maxValue: 40.5, eu: "40", us: "15.75", uk: "15.75" },
  { maxValue: 41.5, eu: "41", us: "16", uk: "16" },
  { maxValue: 42.5, eu: "42", us: "16.5", uk: "16.5" },
  { maxValue: 43.5, eu: "43", us: "17", uk: "17" },
  { maxValue: 999, eu: "44", us: "17.5", uk: "17.5" },
];

// Haut (t-shirt/pull) S/M/L/XL par tour de poitrine (cm). Centres : M≈98, L≈106.
// Sources : underratedclub.com t-shirt sizing ; hemden.de shirt conversion ;
// sizechart.com mens shirt european (S 91-96, M 96-101, L 106-111, XL 111-117).
export const TOP_BANDS: SizeBand[] = [
  { maxValue: 88, eu: "XS", us: "XS", uk: "XS" },
  { maxValue: 96, eu: "S", us: "S", uk: "S" },
  { maxValue: 104, eu: "M", us: "M", uk: "M" },
  { maxValue: 112, eu: "L", us: "L", uk: "L" },
  { maxValue: 120, eu: "XL", us: "XL", uk: "XL" },
  { maxValue: 999, eu: "XXL", us: "XXL", uk: "XXL" },
];

// Pantalon — tour de taille (cm) -> FR (eu) et W pouces (us/uk). Valeurs INDICATIVES
// (le "W" jean varie fortement selon marque / vanity sizing). Sources : asos.com
// mens trousers ; levi.com waist size guide. La correction manuelle pallie l'écart marque.
export const PANTS_WAIST_BANDS: SizeBand[] = [
  { maxValue: 74, eu: "38", us: "W28", uk: "W28" },
  { maxValue: 79, eu: "40", us: "W30", uk: "W30" },
  { maxValue: 84, eu: "42", us: "W32", uk: "W32" },
  { maxValue: 89, eu: "44", us: "W34", uk: "W34" },
  { maxValue: 94, eu: "46", us: "W36", uk: "W36" },
  { maxValue: 99, eu: "48", us: "W38", uk: "W38" },
  { maxValue: 999, eu: "50", us: "W40", uk: "W40" },
];

// Conversion pointure EU -> US/UK (la pointure est SAISIE par l'utilisateur ; cette
// table n'est qu'un confort d'affichage). Valeurs INDICATIVES : les conversions
// chaussures divergent selon les sources/marques. Convention courante UK (US = UK + 0.5).
// Sources : asknumbers.com mens shoe conversion ; size.ly international shoe chart.
export const SHOE_CONVERSIONS: Record<string, { us: string; uk: string }> = {
  "39": { us: "7", uk: "6" },
  "40": { us: "7.5", uk: "6.5" },
  "41": { us: "8", uk: "7" },
  "42": { us: "8.5", uk: "8" },
  "43": { us: "9.5", uk: "9" },
  "44": { us: "10", uk: "9.5" },
  "45": { us: "11", uk: "10.5" },
  "46": { us: "12", uk: "11" },
};
