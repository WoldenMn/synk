import { createHash } from "node:crypto";
import { alphaAt, readPng } from "./png-lib.mjs";

export const PLATE_FILES = {
  landmarks: "src/features/bodyProfile/plate-landmarks.json",
  catalog: "src/features/bodyProfile/catalog.json",
  front: "src/assets/body/front.png",
  back: "src/assets/body/back.png",
  provenance: "src/assets/body/PROVENANCE.md",
};

export const OPAQUE = 128;
export const EDGE_TOLERANCE = 1.5;
export const CROTCH_TOLERANCE = 3;
export const PLATEAU_SLACK = 1;
export const PLATEAU_MARGIN = 4;
export const PLATEAU_MAX_SPAN = 60;
export const ASPECT_TOLERANCE = 0.002;
const STEP = 0.5;
const GAP_WINDOW = 20;

export function loadPlate(read) {
  const landmarks = JSON.parse(read(PLATE_FILES.landmarks).toString("utf-8"));
  const catalog = JSON.parse(read(PLATE_FILES.catalog).toString("utf-8"));
  return {
    viewBox: landmarks.viewBox,
    landmarks: landmarks.y,
    anchors: catalog.measurements
      .filter((m) => m.anchor)
      .map(({ slug, zone, anchor }) => ({ slug, zone, anchor })),
    figures: {
      front: readPng(read(PLATE_FILES.front)),
      back: readPng(read(PLATE_FILES.back)),
    },
  };
}

export function runsAt(png, row) {
  const runs = [];
  let from = -1;
  for (let x = 0; x <= png.width; x++) {
    const inside = x < png.width && alphaAt(png, x, row) >= OPAQUE;
    if (inside && from < 0) from = x;
    if (!inside && from >= 0) {
      runs.push({ from, to: x });
      from = -1;
    }
  }
  return runs;
}

function inkRows(png) {
  let top = -1;
  let bottom = -1;
  for (let y = 0; y < png.height; y++) {
    if (runsAt(png, y).length > 0) {
      if (top < 0) top = y;
      bottom = y + 1;
    }
  }
  return { top, bottom };
}

function plateau(samples, mode) {
  const extreme = Math[mode](...samples.map((s) => s.width));
  const inside = samples.filter((s) => Math.abs(s.width - extreme) <= PLATEAU_SLACK);
  return { from: inside[0].y, to: inside.at(-1).y };
}

function sampleRange(png, scale, fromY, toY, pick) {
  const samples = [];
  for (let y = fromY; y <= toY; y += STEP) {
    const run = pick(runsAt(png, Math.min(png.height - 1, Math.floor(y * scale))), y);
    if (run) samples.push({ y, width: (run.to - run.from) / scale });
  }
  return samples;
}

function legPicker(side) {
  return (runs) => {
    const widest = [...runs].sort((a, b) => b.to - b.from - (a.to - a.from)).slice(0, 2);
    if (widest.length < 2) return null;
    widest.sort((a, b) => a.from - b.from);
    return side === "left" ? widest[0] : widest[1];
  };
}

function findCrotch(png, scale, axis, fromY, toY) {
  const low = (axis - GAP_WINDOW) * scale;
  const high = (axis + GAP_WINDOW) * scale;
  for (let y = fromY; y <= toY; y += STEP) {
    const runs = runsAt(png, Math.min(png.height - 1, Math.floor(y * scale)));
    if (runs.filter((r) => r.to > low && r.from < high).length >= 2) return y;
  }
  return null;
}

export function measureFigure(png, viewBox) {
  const scale = png.width / viewBox.width;
  const { top, bottom } = inkRows(png);
  const first = top / scale;
  const last = bottom / scale;
  const at = (fraction) => first + fraction * (last - first);
  const shoulders = runsAt(png, Math.floor(at(0.2) * scale)).sort(
    (a, b) => b.to - b.from - (a.to - a.from)
  )[0];
  const axis = shoulders ? (shoulders.from + shoulders.to) / 2 / scale : viewBox.width / 2;
  const central = (runs) => runs.find((r) => r.from <= axis * scale && r.to > axis * scale);
  const empty = { from: null, to: null };
  const span = (samples, mode) => (samples.length > 0 ? plateau(samples, mode) : empty);
  return {
    scale,
    top: first,
    bottom: last,
    axis,
    crotch: findCrotch(png, scale, axis, at(0.5), at(0.7)),
    waist: span(sampleRange(png, scale, at(0.3), at(0.5), central), "min"),
    calf: ["left", "right"].map((side) =>
      span(sampleRange(png, scale, at(0.75), at(0.9), legPicker(side)), "max")
    ),
    ankle: ["left", "right"].map((side) =>
      span(sampleRange(png, scale, at(0.85), at(0.95), legPicker(side)), "min")
    ),
  };
}

function plateauProblems(view, name, value, spans) {
  const problems = [];
  for (const [index, { from, to }] of spans.entries()) {
    const where = spans.length > 1 ? ` (jambe ${index + 1})` : "";
    if (from === null) {
      problems.push(`${view} : aucun palier mesurable pour ${name}${where}`);
    } else if (to - from > PLATEAU_MAX_SPAN) {
      problems.push(`${view} : le palier de ${name}${where} est trop large (${from} a ${to})`);
    } else if (value < from - PLATEAU_MARGIN || value > to + PLATEAU_MARGIN) {
      problems.push(
        `${view} : ${name} a ${value} hors du palier mesure ${from} a ${to} (marge ${PLATEAU_MARGIN})`
      );
    }
  }
  return problems;
}

function landmarkProblems(view, measures, landmarks) {
  const problems = [];
  const edge = (name, measured) => {
    if (Math.abs(landmarks[name] - measured) > EDGE_TOLERANCE) {
      problems.push(`${view} : ${name} a ${landmarks[name]}, la figure mesure ${measured}`);
    }
  };
  edge("crown", measures.top);
  edge("sole", measures.bottom);
  if (measures.crotch === null) {
    problems.push(`${view} : aucune fourche mesurable entre les jambes`);
  } else if (Math.abs(landmarks.crotch - measures.crotch) > CROTCH_TOLERANCE) {
    problems.push(
      `${view} : crotch a ${landmarks.crotch}, la fourche se mesure a ${measures.crotch}`
    );
  }
  return [
    ...problems,
    ...plateauProblems(view, "waist", landmarks.waist, [measures.waist]),
    ...plateauProblems(view, "calf", landmarks.calf, measures.calf),
    ...plateauProblems(view, "ankle", landmarks.ankle, measures.ankle),
  ];
}

function anchorProblems({ viewBox, figures, anchors }) {
  const problems = [];
  for (const { slug, zone, anchor } of anchors) {
    const png = figures[anchor.view];
    if (!png) {
      problems.push(`${slug} : vue inconnue ${anchor.view}`);
      continue;
    }
    const scale = png.width / viewBox.width;
    const x = Math.round((anchor.x / 100) * viewBox.width * scale);
    const y = Math.round((1 - anchor.y / 100) * viewBox.height * scale);
    if (alphaAt(png, x, y) < OPAQUE) {
      problems.push(`${slug} : l'ancre (${anchor.x} %, ${anchor.y} %) tombe hors du corps`);
      continue;
    }
    if (zone === "arms") {
      const runs = runsAt(png, y);
      const index = runs.findIndex((r) => x >= r.from && x < r.to);
      if (runs.length < 3 || (index !== 0 && index !== runs.length - 1)) {
        problems.push(`${slug} : l'ancre d'un bras ne tombe pas sur un bras detache du tronc`);
      }
    }
  }
  return problems;
}

const REQUIRED = ["crown", "waist", "crotch", "calf", "ankle", "sole"];

export function plateProblems({ viewBox, landmarks, figures, anchors }) {
  const missing = REQUIRED.filter((name) => typeof landmarks[name] !== "number");
  if (missing.length > 0) return [`reperes absents ou illisibles : ${missing.join(", ")}`];
  const problems = [];
  const names = Object.keys(landmarks);
  names.slice(1).forEach((name, i) => {
    if (landmarks[name] <= landmarks[names[i]]) {
      problems.push(`reperes : ${name} (${landmarks[name]}) ne descend pas sous ${names[i]}`);
    }
  });
  const sizes = new Set(Object.values(figures).map((p) => `${p.width}x${p.height}`));
  if (sizes.size > 1) problems.push(`les deux vues n'ont pas la meme taille : ${[...sizes]}`);
  for (const [view, png] of Object.entries(figures)) {
    const ratio = png.width / png.height - viewBox.width / viewBox.height;
    if (Math.abs(ratio) > ASPECT_TOLERANCE) {
      problems.push(`${view} : proportions de l'image differentes du viewBox (ecart ${ratio})`);
      continue;
    }
    problems.push(...landmarkProblems(view, measureFigure(png, viewBox), landmarks));
  }
  return [...problems, ...anchorProblems({ viewBox, figures, anchors })];
}

export function provenanceProblems(markdown, files) {
  return Object.entries(files)
    .map(([name, buffer]) => [name, createHash("sha256").update(buffer).digest("hex")])
    .filter(([, hash]) => !markdown.includes(hash))
    .map(([name, hash]) => `${PLATE_FILES.provenance} ne cite pas le SHA-256 de ${name} (${hash})`);
}
