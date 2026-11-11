// Genere toutes les icones de Synk depuis les trois SVG de src-tauri/icons/source.
//
// Une seule marque, trois dessins : le grand (48 px et plus) porte l'anneau de
// synchronisation et le trace de pouls ; celui de 32 px les redessine sur la
// grille de 32 ; celui de 16 px ne garde que le pouls, seul lisible a cette
// taille. Le .ico assemble ici prend chaque taille dans le dessin qui lui est
// destine, la ou `tauri icon` les deduirait toutes du grand. Les deux bitmaps
// de l'installeur NSIS sont composes depuis le grand dessin (nsis-images.mjs).
//
// Usage : npm run icons:gen

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { decodePng, header, sidebar } from "./nsis-images.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE = path.join(ROOT, "src-tauri", "icons", "source");
const OUT = path.join(ROOT, "src-tauri", "icons");
const PUBLIC = path.join(ROOT, "public");
const TAURI = path.join(ROOT, "node_modules", "@tauri-apps", "cli", "tauri.js");

// Taille du cadre .ico -> dessin source.
const ICO_FRAMES = [
  [16, "synk-icon-16.svg"],
  [20, "synk-icon-16.svg"],
  [24, "synk-icon-16.svg"],
  [32, "synk-icon-32.svg"],
  [40, "synk-icon-32.svg"],
  [48, "synk-icon.svg"],
  [64, "synk-icon.svg"],
  [256, "synk-icon.svg"],
];

function tauriIcon(svgFile, outDir, sizes = []) {
  const args = [TAURI, "icon", svgFile, "-o", outDir];
  for (const s of sizes) args.push("-p", String(s));
  try {
    execFileSync(process.execPath, args, { stdio: ["ignore", "ignore", "pipe"] });
  } catch (e) {
    const name = path.basename(svgFile);
    throw new Error(`tauri icon ${name} a echoue : ${e.stderr?.toString() ?? e.message}`, {
      cause: e,
    });
  }
}

function renderPng(svgFile, size, tmp) {
  const dir = path.join(tmp, `${path.basename(svgFile, ".svg")}-${size}`);
  tauriIcon(svgFile, dir, [size]);
  const label = `${path.basename(svgFile)} @${size}`;
  return decodePng(fs.readFileSync(path.join(dir, `${size}x${size}.png`)), label);
}

// La marque seule (anneau et pouls blancs) : le grand dessin sans sa tuile.
function markSvg(tmp) {
  const svg = fs.readFileSync(path.join(SOURCE, "synk-icon.svg"), "utf8");
  const mark = svg.replace(/<rect\b[^>]*\/>/, "");
  if (mark === svg) throw new Error("synk-icon.svg : tuile <rect> introuvable");
  const file = path.join(tmp, "synk-mark.svg");
  fs.writeFileSync(file, mark);
  return file;
}

function pngSize(buf, label) {
  const sig = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
  if (!sig.every((b, i) => buf[i] === b)) throw new Error(`${label} n'est pas un PNG`);
  return [buf.readUInt32BE(16), buf.readUInt32BE(20)];
}

// ICO a entrees PNG (Windows Vista et suivants) : en-tete, repertoire, images.
function buildIco(frames) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(frames.length, 4);
  const dir = Buffer.alloc(16 * frames.length);
  let offset = header.length + dir.length;
  frames.forEach(({ size, png }, i) => {
    const e = i * 16;
    dir.writeUInt8(size >= 256 ? 0 : size, e);
    dir.writeUInt8(size >= 256 ? 0 : size, e + 1);
    dir.writeUInt8(0, e + 2);
    dir.writeUInt8(0, e + 3);
    dir.writeUInt16LE(1, e + 4);
    dir.writeUInt16LE(32, e + 6);
    dir.writeUInt32LE(png.length, e + 8);
    dir.writeUInt32LE(offset, e + 12);
    offset += png.length;
  });
  return Buffer.concat([header, dir, ...frames.map((f) => f.png)]);
}

function main() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "synk-icons-"));
  try {
    const full = path.join(tmp, "full");
    tauriIcon(path.join(SOURCE, "synk-icon.svg"), full);
    // Tout le jeu standard, rendu depuis le grand dessin ; les dossiers android/
    // et ios/ sont ignores (cible Windows seule). icon.ico est remplace plus bas
    // par l'assemblage par taille.
    const standard = fs
      .readdirSync(full, { withFileTypes: true })
      .filter((e) => e.isFile() && e.name !== "icon.ico")
      .map((e) => e.name);
    for (const name of standard) fs.copyFileSync(path.join(full, name), path.join(OUT, name));

    const bySource = new Map();
    for (const [size, svg] of ICO_FRAMES) {
      if (!bySource.has(svg)) bySource.set(svg, []);
      bySource.get(svg).push(size);
    }
    const frames = [];
    for (const [svg, sizes] of bySource) {
      const dir = path.join(tmp, path.basename(svg, ".svg"));
      tauriIcon(path.join(SOURCE, svg), dir, sizes);
      for (const size of sizes) {
        const png = fs.readFileSync(path.join(dir, `${size}x${size}.png`));
        const [w, h] = pngSize(png, `${svg} @${size}`);
        if (w !== size || h !== size) throw new Error(`${svg} @${size} rendu en ${w}x${h}`);
        frames.push({ size, png });
      }
    }
    frames.sort((a, b) => a.size - b.size);
    fs.writeFileSync(path.join(OUT, "icon.ico"), buildIco(frames));

    const mark = renderPng(markSvg(tmp), 132, tmp);
    const tile = renderPng(path.join(SOURCE, "synk-icon.svg"), 48, tmp);
    fs.writeFileSync(path.join(OUT, "nsis-sidebar.bmp"), sidebar(mark));
    fs.writeFileSync(path.join(OUT, "nsis-header.bmp"), header(tile));

    fs.mkdirSync(PUBLIC, { recursive: true });
    fs.copyFileSync(path.join(SOURCE, "synk-icon.svg"), path.join(PUBLIC, "synk.svg"));

    console.log(`icon.ico : ${frames.map((f) => f.size).join(", ")} px`);
    console.log(`${standard.length} fichiers du jeu standard ; favicon : public/synk.svg`);
    console.log("installeur : nsis-sidebar.bmp (164 x 314), nsis-header.bmp (150 x 57)");
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

main();
