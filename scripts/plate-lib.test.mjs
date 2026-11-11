import { readFileSync, statSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { encodePng } from "./png-fixture.mjs";
import { readPng } from "./png-lib.mjs";
import {
  PLATE_FILES,
  loadPlate,
  measureFigure,
  plateProblems,
  provenanceProblems,
  runsAt,
} from "./plate-lib.mjs";

const read = (path) => readFileSync(path);
const REAL = loadPlate(read);
const SCALE = 2;
const AXIS = 128;

function legWidth(y) {
  if (y < 500) return 60 - ((y - 396) * 20) / 104;
  if (y < 550) return 40 + ((y - 500) * 4) / 50;
  if (y < 628) return 44 - ((y - 550) * 20) / 78;
  return 24 + ((y - 628) * 26) / 62;
}

function torsoHalf(y) {
  return y < 272 ? 62 - ((y - 230) * 12) / 42 : 50 + ((y - 272) * 20) / 124;
}

function spansAt(y) {
  if (y < 21 || y >= 690) return [];
  if (y < 100) return [[AXIS - 30, AXIS + 30]];
  if (y < 130) return [[AXIS - 22, AXIS + 22]];
  if (y < 150) {
    const half = 22 + (y - 130) * 4.5;
    return [[AXIS - half, AXIS + half]];
  }
  if (y < 230) return [[AXIS - 112, AXIS + 112]];
  if (y < 396) {
    const half = torsoHalf(y);
    return [
      [AXIS - 112, AXIS - 76],
      [AXIS - half, AXIS + half],
      [AXIS + 76, AXIS + 112],
    ];
  }
  const width = legWidth(y);
  return [
    [AXIS - 6 - width, AXIS - 6],
    [AXIS + 6, AXIS + 6 + width],
  ];
}

function syntheticFigure(width = 512, height = 1400) {
  const pixels = new Uint8Array(width * height * 2);
  for (let row = 0; row < height; row++) {
    for (const [from, to] of spansAt(row / SCALE)) {
      for (let x = Math.ceil(from * SCALE); x < Math.floor(to * SCALE); x++) {
        pixels[(row * width + x) * 2] = 200;
        pixels[(row * width + x) * 2 + 1] = 255;
      }
    }
  }
  return readPng(encodePng({ width, height, channels: 2, pixels }));
}

const BOX = REAL.viewBox;
const GOOD_LANDMARKS = REAL.landmarks;
const ARM_ANCHOR = {
  slug: "bicep_left",
  zone: "arms",
  anchor: { view: "front", x: ((AXIS - 94) / 256) * 100, y: (1 - 300 / 700) * 100 },
};
const synthetic = () => ({
  viewBox: BOX,
  landmarks: GOOD_LANDMARKS,
  figures: { front: syntheticFigure(), back: syntheticFigure() },
  anchors: [ARM_ANCHOR],
});

describe("measureFigure, sur une figure dessinee dont on connait les cotes", () => {
  const measures = measureFigure(syntheticFigure(), BOX);

  it("trouve le sommet, la plante, l'axe et la fourche", () => {
    expect(measures.top).toBe(21);
    expect(measures.bottom).toBe(690);
    expect(Math.abs(measures.axis - AXIS)).toBeLessThanOrEqual(1);
    expect(Math.abs(measures.crotch - 396)).toBeLessThanOrEqual(0.5);
  });

  it("trouve le palier de la taille, du mollet et de la cheville", () => {
    expect(measures.waist.from).toBeLessThanOrEqual(272);
    expect(measures.waist.to).toBeGreaterThanOrEqual(272);
    for (const calf of measures.calf) {
      expect(calf.from).toBeLessThanOrEqual(550);
      expect(calf.to).toBeGreaterThanOrEqual(550);
    }
    for (const ankle of measures.ankle) {
      expect(ankle.from).toBeLessThanOrEqual(628);
      expect(ankle.to).toBeGreaterThanOrEqual(628);
    }
  });
});

describe("plateProblems, cas connu bon et cas connus mauvais sur la figure dessinee", () => {
  it("ne dit rien quand tout est en place", () => {
    expect(plateProblems(synthetic())).toEqual([]);
  });

  it.each([
    ["le sommet du crane", { crown: 30 }, /crown a 30/],
    ["la plante des pieds", { sole: 680 }, /sole a 680/],
    ["la fourche", { crotch: 410 }, /crotch a 410/],
    ["la taille", { waist: 300 }, /waist a 300 hors du palier/],
    ["le mollet", { calf: 520 }, /calf a 520 hors du palier/],
    ["la cheville", { ankle: 600 }, /ankle a 600 hors du palier/],
  ])("voit %s deplace", (_name, change, expected) => {
    const plate = synthetic();
    plate.landmarks = { ...plate.landmarks, ...change };
    expect(plateProblems(plate).join("\n")).toMatch(expected);
  });

  it("voit des reperes qui ne descendent plus de haut en bas", () => {
    const plate = synthetic();
    plate.landmarks = { ...plate.landmarks, chin: 90 };
    expect(plateProblems(plate).join("\n")).toMatch(/chin \(90\) ne descend pas sous jaw/);
  });

  it("voit un repere obligatoire qui manque", () => {
    const plate = synthetic();
    plate.landmarks = { ...plate.landmarks };
    delete plate.landmarks.waist;
    expect(plateProblems(plate)).toEqual(["reperes absents ou illisibles : waist"]);
  });

  it("voit une image aux proportions du viewBox rompues, ou deux vues de tailles differentes", () => {
    const plate = synthetic();
    plate.figures = { front: syntheticFigure(512, 1400), back: syntheticFigure(400, 1400) };
    const text = plateProblems(plate).join("\n");
    expect(text).toMatch(/pas la meme taille/);
    expect(text).toMatch(/back : proportions de l'image/);
  });

  it("voit une figure sans taille marquee", () => {
    const tube = readPng(
      encodePng({
        width: 512,
        height: 1400,
        channels: 2,
        pixels: Uint8Array.from({ length: 512 * 1400 * 2 }, (_, i) => (i % 2 === 0 ? 200 : 255)),
      })
    );
    const plate = synthetic();
    plate.figures = { front: tube, back: tube };
    expect(plateProblems(plate).join("\n")).toMatch(/palier de waist est trop large/);
  });

  it("voit une ancre hors du corps et une ancre de bras posee sur le tronc ou sur des bras joints", () => {
    const plate = synthetic();
    const outside = { slug: "wrist_left", zone: "arms", anchor: { view: "front", x: 2, y: 50 } };
    const onTorso = {
      slug: "bicep_left",
      zone: "arms",
      anchor: { ...ARM_ANCHOR.anchor, x: (AXIS / 256) * 100 },
    };
    const merged = {
      slug: "arm_length_left",
      zone: "arms",
      anchor: { ...ARM_ANCHOR.anchor, y: (1 - 200 / 700) * 100 },
    };
    plate.anchors = [outside, onTorso, merged];
    const text = plateProblems(plate).join("\n");
    expect(text).toMatch(/wrist_left : l'ancre \(2 %, 50 %\) tombe hors du corps/);
    expect(text).toMatch(/bicep_left : l'ancre d'un bras ne tombe pas sur un bras detache/);
    expect(text).toMatch(/arm_length_left : l'ancre d'un bras ne tombe pas sur un bras detache/);
  });

  it("voit une vue d'ancre inconnue", () => {
    const plate = synthetic();
    plate.anchors = [{ slug: "x", zone: "torso", anchor: { view: "side", x: 50, y: 50 } }];
    expect(plateProblems(plate)).toEqual(["x : vue inconnue side"]);
  });
});

describe("les figures livrees, lues par le meme instrument", () => {
  it("donnent les memes plages opaques qu'une lecture externe (Pillow)", () => {
    const expected = {
      front: {
        402: [[72, 440]],
        522: [
          [58, 115],
          [152, 364],
          [392, 449],
        ],
        802: [
          [61, 247],
          [270, 400],
          [406, 461],
        ],
        1000: [
          [123, 202],
          [315, 394],
        ],
      },
      back: {
        402: [[58, 443]],
        522: [
          [52, 116],
          [141, 357],
          [386, 453],
        ],
        802: [
          [49, 242],
          [263, 453],
        ],
        1000: [
          [115, 204],
          [307, 395],
        ],
      },
    };
    for (const [view, rows] of Object.entries(expected)) {
      for (const [row, runs] of Object.entries(rows)) {
        const found = runsAt(REAL.figures[view], Number(row)).map((r) => [r.from, r.to]);
        expect(found, `${view}, ligne ${row}`).toEqual(runs);
      }
    }
  });

  it("ont des reperes et des ancres coherents avec le corps dessine", () => {
    expect(plateProblems(REAL)).toEqual([]);
  });

  it("voient une ancre de bras glissee sur le tronc ou un repere decale (cas mauvais sur la vraie figure)", () => {
    const biceps = REAL.anchors.find((a) => a.slug === "bicep_left");
    const onTorso = { ...biceps, anchor: { ...biceps.anchor, x: 50 } };
    const moved = REAL.anchors.map((a) => (a.slug === "bicep_left" ? onTorso : a));
    expect(plateProblems({ ...REAL, anchors: moved }).join("\n")).toMatch(/bicep_left/);
    const landmarks = { ...REAL.landmarks, waist: 330, ankle: 560 };
    const text = plateProblems({ ...REAL, landmarks }).join("\n");
    expect(text).toMatch(/waist a 330 hors du palier/);
    expect(text).toMatch(/ankle a 560 hors du palier/);
  });

  it("tiennent sous 150 Ko par vue", () => {
    for (const file of [PLATE_FILES.front, PLATE_FILES.back]) {
      expect(statSync(file).size).toBeLessThan(150 * 1024);
    }
  });

  it("sont citees avec leur SHA-256 dans PROVENANCE.md", () => {
    const markdown = read(PLATE_FILES.provenance).toString("utf-8");
    const files = { "front.png": read(PLATE_FILES.front), "back.png": read(PLATE_FILES.back) };
    expect(provenanceProblems(markdown, files)).toEqual([]);
    expect(provenanceProblems("rien de cite", files)).toHaveLength(2);
  });
});
