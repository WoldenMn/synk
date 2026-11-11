import { crc32, deflateSync } from "node:zlib";

const COLOR_TYPE = { 1: 0, 2: 4, 3: 2, 4: 6 };

function paethPredictor(a, b, c) {
  const p = a + b - c;
  const best = Math.min(Math.abs(p - a), Math.abs(p - b), Math.abs(p - c));
  if (Math.abs(p - a) === best) return a;
  if (Math.abs(p - b) === best) return b;
  return c;
}

function chunk(type, data) {
  const head = Buffer.alloc(8);
  head.writeUInt32BE(data.length, 0);
  head.write(type, 4, "latin1");
  const tail = Buffer.alloc(4);
  tail.writeUInt32BE(crc32(Buffer.concat([head.subarray(4), data])), 0);
  return Buffer.concat([head, data, tail]);
}

function predicted(filter, a, b, c) {
  if (filter === 1) return a;
  if (filter === 2) return b;
  if (filter === 3) return Math.floor((a + b) / 2);
  if (filter === 4) return paethPredictor(a, b, c);
  return 0;
}

export function encodePng({ width, height, channels, pixels, filter = 0 }) {
  const stride = width * channels;
  const rows = [];
  for (let y = 0; y < height; y++) {
    const row = Buffer.alloc(stride + 1);
    row[0] = filter;
    for (let x = 0; x < stride; x++) {
      const raw = pixels[y * stride + x];
      const a = x >= channels ? pixels[y * stride + x - channels] : 0;
      const b = y > 0 ? pixels[(y - 1) * stride + x] : 0;
      const c = y > 0 && x >= channels ? pixels[(y - 1) * stride + x - channels] : 0;
      row[x + 1] = (raw - predicted(filter, a, b, c)) & 255;
    }
    rows.push(row);
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header[8] = 8;
  header[9] = COLOR_TYPE[channels];
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", header),
    chunk("IDAT", deflateSync(Buffer.concat(rows))),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}
