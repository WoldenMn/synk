import { inflateSync } from "node:zlib";

const SIGNATURE = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
const CHANNELS = { 0: 1, 2: 3, 4: 2, 6: 4 };

function paeth(left, up, upLeft) {
  const estimate = left + up - upLeft;
  const dLeft = Math.abs(estimate - left);
  const dUp = Math.abs(estimate - up);
  const dUpLeft = Math.abs(estimate - upLeft);
  if (dLeft <= dUp && dLeft <= dUpLeft) return left;
  return dUp <= dUpLeft ? up : upLeft;
}

function chunksOf(buffer) {
  const chunks = [];
  let offset = SIGNATURE.length;
  while (offset + 12 <= buffer.length) {
    const length = buffer.readUInt32BE(offset);
    const type = buffer.toString("latin1", offset + 4, offset + 8);
    const end = offset + 8 + length;
    if (end + 4 > buffer.length) throw new Error(`bloc PNG ${type} tronque`);
    chunks.push({ type, data: buffer.subarray(offset + 8, end) });
    offset = end + 4;
    if (type === "IEND") break;
  }
  return chunks;
}

function unfilter(raw, width, height, bytesPerPixel) {
  const stride = width * bytesPerPixel;
  if (raw.length !== height * (stride + 1)) throw new Error("donnees PNG de taille inattendue");
  const out = new Uint8Array(height * stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const source = y * (stride + 1) + 1;
    const target = y * stride;
    for (let x = 0; x < stride; x++) {
      const left = x >= bytesPerPixel ? out[target + x - bytesPerPixel] : 0;
      const up = y > 0 ? out[target - stride + x] : 0;
      const upLeft = y > 0 && x >= bytesPerPixel ? out[target - stride + x - bytesPerPixel] : 0;
      const value = raw[source + x];
      if (filter === 0) out[target + x] = value;
      else if (filter === 1) out[target + x] = value + left;
      else if (filter === 2) out[target + x] = value + up;
      else if (filter === 3) out[target + x] = value + ((left + up) >> 1);
      else if (filter === 4) out[target + x] = value + paeth(left, up, upLeft);
      else throw new Error(`filtre PNG inconnu : ${filter}`);
    }
  }
  return out;
}

export function readPng(buffer) {
  if (buffer.length < SIGNATURE.length || !buffer.subarray(0, SIGNATURE.length).equals(SIGNATURE)) {
    throw new Error("signature PNG absente");
  }
  const chunks = chunksOf(buffer);
  const header = chunks.find((c) => c.type === "IHDR");
  if (!header) throw new Error("en-tete PNG absent");
  const width = header.data.readUInt32BE(0);
  const height = header.data.readUInt32BE(4);
  const depth = header.data[8];
  const colorType = header.data[9];
  const interlace = header.data[12];
  const channels = CHANNELS[colorType];
  if (depth !== 8 || channels === undefined || interlace !== 0) {
    throw new Error(
      `PNG non pris en charge (profondeur ${depth}, type ${colorType}, entrelacement ${interlace})`
    );
  }
  const compressed = Buffer.concat(chunks.filter((c) => c.type === "IDAT").map((c) => c.data));
  const raw = inflateSync(compressed);
  return { width, height, channels, pixels: unfilter(raw, width, height, channels) };
}

export function alphaAt(png, x, y) {
  if (x < 0 || y < 0 || x >= png.width || y >= png.height) return 0;
  const base = (y * png.width + x) * png.channels;
  if (png.channels === 2) return png.pixels[base + 1];
  if (png.channels === 4) return png.pixels[base + 3];
  return 255;
}
