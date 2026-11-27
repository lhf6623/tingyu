import zlib from "node:zlib";
import fs from "node:fs";

const S = 64;
const RED = [200, 68, 60];
const WHITE = [255, 255, 255];

function inCircle(x, y, cx, cy, r) {
  return (x - cx) ** 2 + (y - cy) ** 2 <= r * r;
}

function inTriangle(x, y) {
  const ax = 26, ay = 20, bx = 26, by = 44, cx = 46, cy = 32;
  const d1 = (x - bx) * (ay - by) - (ax - bx) * (y - by);
  const d2 = (x - cx) * (by - cy) - (bx - cx) * (y - cy);
  const d3 = (x - ax) * (cy - ay) - (cx - ax) * (y - ay);
  const neg = d1 < 0 || d2 < 0 || d3 < 0;
  const pos = d1 > 0 || d2 > 0 || d3 > 0;
  return !(neg && pos);
}

// 3x3 超采样做抗锯齿，托盘图标小，边缘毛糙会很难看。
const SS = 3;
const raw = Buffer.alloc((S * 4 + 1) * S);

for (let y = 0; y < S; y++) {
  const row = y * (S * 4 + 1);
  raw[row] = 0;
  for (let x = 0; x < S; x++) {
    let circleCover = 0;
    let whiteCover = 0;
    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const px = x + (sx + 0.5) / SS;
        const py = y + (sy + 0.5) / SS;
        if (inCircle(px, py, 32, 32, 30)) {
          circleCover += 1;
          if (inTriangle(px, py)) whiteCover += 1;
        }
      }
    }
    const total = SS * SS;
    const alpha = circleCover / total;
    const p = row + 1 + x * 4;
    if (alpha === 0) {
      raw[p] = 0;
      raw[p + 1] = 0;
      raw[p + 2] = 0;
      raw[p + 3] = 0;
      continue;
    }
    const white = whiteCover / Math.max(circleCover, 1);
    raw[p] = Math.round(RED[0] * (1 - white) + WHITE[0] * white);
    raw[p + 1] = Math.round(RED[1] * (1 - white) + WHITE[1] * white);
    raw[p + 2] = Math.round(RED[2] * (1 - white) + WHITE[2] * white);
    raw[p + 3] = Math.round(alpha * 255);
  }
}

const crcTable = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(buf) {
  let c = -1;
  for (const byte of buf) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([len, body, crc]);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(S, 0);
ihdr.writeUInt32BE(S, 4);
ihdr[8] = 8;
ihdr[9] = 6;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

fs.writeFileSync("src-tauri/icons/tray.png", png);
console.log("src-tauri/icons/tray.png", png.length, "bytes");
