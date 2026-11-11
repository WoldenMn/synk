// Bitmaps de l'installeur NSIS, composes depuis des rendus PNG du SVG maitre :
// barre laterale des pages Bienvenue et Fin (164 x 314), bandeau des autres
// pages (150 x 57). NSIS n'affiche que du BMP sans transparence, d'ou le
// decodage PNG et l'aplatissement sur un fond ici.

import zlib from "node:zlib";

export const SIDEBAR = { width: 164, height: 314 };
export const HEADER = { width: 150, height: 57 };

const TILE_TOP = [0x63, 0x66, 0xf1];
const TILE_BOTTOM = [0x43, 0x38, 0xca];
const WHITE = [0xff, 0xff, 0xff];
const BAYER = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

// PNG 8 bits, RGB ou RGBA, non entrelace : ce que rend `tauri icon`.
export function decodePng(buf, label) {
  let offset = 8;
  let header = null;
  const data = [];
  while (offset < buf.length) {
    const length = buf.readUInt32BE(offset);
    const type = buf.toString("latin1", offset + 4, offset + 8);
    const body = buf.subarray(offset + 8, offset + 8 + length);
    if (type === "IHDR") header = body;
    if (type === "IDAT") data.push(body);
    offset += 12 + length;
  }
  if (!header) throw new Error(`${label} : pas d'en-tete IHDR`);
  const width = header.readUInt32BE(0);
  const height = header.readUInt32BE(4);
  const [depth, colorType, , , interlace] = header.subarray(8, 13);
  const channels = { 2: 3, 6: 4 }[colorType];
  if (depth !== 8 || !channels || interlace !== 0) {
    throw new Error(`${label} : PNG non pris en charge (profondeur ${depth}, type ${colorType})`);
  }
  const raw = zlib.inflateSync(Buffer.concat(data));
  const stride = width * channels;
  const rows = Buffer.alloc(stride * height);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let i = 0; i < stride; i++) {
      const left = i >= channels ? rows[y * stride + i - channels] : 0;
      const up = y > 0 ? rows[(y - 1) * stride + i] : 0;
      const corner = y > 0 && i >= channels ? rows[(y - 1) * stride + i - channels] : 0;
      rows[y * stride + i] = (line[i] + predict(filter, left, up, corner, label)) & 0xff;
    }
  }
  const rgba = Buffer.alloc(width * height * 4, 0xff);
  for (let p = 0; p < width * height; p++)
    rows.copy(rgba, p * 4, p * channels, p * channels + channels);
  return { width, height, rgba };
}

function predict(filter, left, up, corner, label) {
  if (filter === 0) return 0;
  if (filter === 1) return left;
  if (filter === 2) return up;
  if (filter === 3) return (left + up) >> 1;
  if (filter === 4) {
    const p = left + up - corner;
    const [pa, pb, pc] = [Math.abs(p - left), Math.abs(p - up), Math.abs(p - corner)];
    return pa <= pb && pa <= pc ? left : pb <= pc ? up : corner;
  }
  throw new Error(`${label} : filtre PNG ${filter} inconnu`);
}

// Image en flottants RGB, fond donne par background(x, y), puis images posees.
function compose({ width, height }, background, layers) {
  const pixels = new Float64Array(width * height * 3);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) pixels.set(background(x, y), (y * width + x) * 3);
  }
  for (const { image, left, top } of layers) {
    for (let y = 0; y < image.height; y++) {
      for (let x = 0; x < image.width; x++) {
        const [tx, ty] = [left + x, top + y];
        if (tx < 0 || ty < 0 || tx >= width || ty >= height) continue;
        const s = (y * image.width + x) * 4;
        const alpha = image.rgba[s + 3] / 255;
        const d = (ty * width + tx) * 3;
        for (let c = 0; c < 3; c++) {
          pixels[d + c] = image.rgba[s + c] * alpha + pixels[d + c] * (1 - alpha);
        }
      }
    }
  }
  return { width, height, pixels };
}

// BMP 24 bits, lignes de bas en haut ; tramage ordonne 4 x 4 contre les paliers du degrade.
export function encodeBmp({ width, height, pixels }) {
  const stride = Math.ceil((width * 3) / 4) * 4;
  const out = Buffer.alloc(54 + stride * height);
  out.write("BM", 0, "latin1");
  out.writeUInt32LE(out.length, 2);
  out.writeUInt32LE(54, 10);
  out.writeUInt32LE(40, 14);
  out.writeInt32LE(width, 18);
  out.writeInt32LE(height, 22);
  out.writeUInt16LE(1, 26);
  out.writeUInt16LE(24, 28);
  out.writeUInt32LE(stride * height, 34);
  out.writeInt32LE(2835, 38);
  out.writeInt32LE(2835, 42);
  for (let y = 0; y < height; y++) {
    const row = 54 + (height - 1 - y) * stride;
    for (let x = 0; x < width; x++) {
      const d = (y * width + x) * 3;
      const threshold = (BAYER[(y & 3) * 4 + (x & 3)] + 0.5) / 16 - 0.5;
      for (let c = 0; c < 3; c++) {
        const value = Math.round(pixels[d + c] + threshold);
        out[row + x * 3 + (2 - c)] = Math.min(255, Math.max(0, value));
      }
    }
  }
  return out;
}

function verticalGradient(height, top, bottom) {
  return (_x, y) => top.map((t, c) => t + ((bottom[c] - t) * y) / (height - 1));
}

// Barre laterale : le degrade de la tuile, la marque blanche (anneau et pouls) en haut.
export function sidebar(mark) {
  const left = Math.round((SIDEBAR.width - mark.width) / 2);
  const top = 110 - Math.round(mark.height / 2);
  const background = verticalGradient(SIDEBAR.height, TILE_TOP, TILE_BOTTOM);
  return encodeBmp(compose(SIDEBAR, background, [{ image: mark, left, top }]));
}

// Bandeau : fond blanc comme l'en-tete NSIS, tuile calee a droite contre le titre.
export function header(tile) {
  const left = HEADER.width - tile.width - 4;
  const top = Math.floor((HEADER.height - tile.height) / 2);
  return encodeBmp(compose(HEADER, () => WHITE, [{ image: tile, left, top }]));
}
