import { readFileSync } from "node:fs";
import {
  PLATE_FILES,
  loadPlate,
  measureFigure,
  plateProblems,
  provenanceProblems,
} from "./plate-lib.mjs";

const read = (path) => readFileSync(path);
const plate = loadPlate(read);

if (process.argv.includes("--mesures")) {
  for (const [view, png] of Object.entries(plate.figures)) {
    console.log(view, JSON.stringify(measureFigure(png, plate.viewBox)));
  }
}

const problems = [
  ...plateProblems(plate),
  ...provenanceProblems(read(PLATE_FILES.provenance).toString("utf-8"), {
    "front.png": read(PLATE_FILES.front),
    "back.png": read(PLATE_FILES.back),
  }),
];
for (const problem of problems) console.error(`[plate] ${problem}`);
if (problems.length === 0) console.log("[plate] figures, reperes et ancres coherents");
process.exit(problems.length === 0 ? 0 : 1);
