import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { alphaAt, readPng } from "./png-lib.mjs";
import { encodePng } from "./png-fixture.mjs";

const REFERENCES = [
  {
    name: "PIL, 2 canaux, 11x9, filtre 1/4",
    channels: 2,
    width: 11,
    height: 9,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAsAAAAJCAQAAABObR6mAAAAkklEQVR42gXBSXLCQBAEwFB4btNLVS8jwkIy3Pz/b/ATrnyAzO3nn5zoQJUaA6SzMY6HeyXEUYylnubFGNcNiQrV7qlrd1QAMu5/WSoVbJGmLbJMz3FcoWiGIzMlyw+Hx3jeNeDE4qx1udt6qmCcv9gBRLqdbQ+iAo7t9clk2GQ7Oj0g6JTtbbag2Tb3YIusBhFfEJIQfWC8HJsAAAAASUVORK5CYII=",
    pixels:
      "032814391f49345b446f587c668d789d89ae98bfadcf1e482d57416a51766086749786ad93bca6cab5dec6f03c604c735c876e947ba990b49dcab4d9c4edd6fde609587f6b93789f8cb19dc6a9d2bee3ccf9dd0af118fe26739d85aa95bfa6d1b5e1c8f4db00ee14fd250c351b4794b9a1cbb1dac2ead8fbe30ff91e082d16432d503960b1d3c1ead1fae10cf41b022c173a284b385f4a6f597fcbf3de06ef18fd230e381d48325b416d5179618e749aea0ef82408311b462951406351785d84739983aa93bc",
  },
  {
    name: "PIL, 4 canaux, 8x7, filtre 1/4",
    channels: 4,
    width: 8,
    height: 7,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAgAAAAHCAYAAAA1WQxeAAAAw0lEQVR42gXBS1KEMBAAUGEigSSdTucfQmBgSqyi3HgMT+S5PJEHcTG+1/19fn0b3fI8DvSaar+BfAxOPHWmloACu5fr3PXEM4rkZYBbXsIKAtycPd2MYsfiUkTtzeQdlNp5Lg+/AEOlrK3hYHu9GmozYGEt8rnHGJcxqveosU/BbAyKeeyoiQP104opSE3BWpJmI28xso+3qqTOqwahKtZplGeQ0bV2CrJ87rqf3+c1jFIQIRTyy2wMM2JLdmX1Ba36B/J7FUEiQzC8AAAAAElFTkSuQmCC",
    pixels:
      "fb3149770b3f668f25476ca02a5e87a2496b92c54f7e9cc45d96ade174a3bef61b4a739d2c587ca4446786bb58729cc85b8ab8dd7997c2ea8cb1dafe9db4ea0a3d6686b4537594c8637ea8db7097c3dc849eceff98badb03a7c6e715b9e1ff2c5e81b0d16d8fc0d77ca8c4f892afdefaa1c5f41ba9db002abfe90f2cd6fe1f4b6b9ac0f48ea9ce0595b6df079ed4ee1eb6df022ccbf1123dd601314eea13406497bfdb00a2cdf31eb0dafd2acbe91833d3f42848de0a3b65fb2745760d2e5f77b1dfff2ab7e70a34c8f81941e1092d5dfb193d610b235c781d4160931d50729f",
  },
  {
    name: "ecrit a la main en Python, 2 canaux, 6x5, filtre 0",
    channels: 2,
    width: 6,
    height: 5,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAYAAAAFCAQAAADMUVVtAAAATElEQVR42gFBAL7/ABAWGEMyNixkT2lLawAqUC5GOmo+flWbdJ4AQGBHfFODcImHoZfKAFtteXx0lYKxqcan1ACDpIy3hs+n267az+zNaRxahIVHsQAAAABJRU5ErkJggg==",
    pixels:
      "1016184332362c644f694b6b2a502e463a6a3e7e559b749e4060477c5383708987a197ca5b6d797c749582b1a9c6a7d483a48cb786cfa7dbaedacfec",
  },
  {
    name: "ecrit a la main en Python, 4 canaux, 6x5, filtre 1",
    channels: 4,
    width: 6,
    height: 5,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAYAAAAFCAYAAABmWJ3mAAAAiElEQVR42gF9AIL/AQg3WYAVEwAAEQotLfz5Bf0PEBcmCjIIBQEwO4CBCwvtE/wcNBAiFgc1AB4DFBcPA+4BQ1+QwfkQC/wsE/4HFQUpE/8j/Az/BhoTAWGDrNX3HSADHgLuGBX8JgUEIP0KDB8qHAGDlMz4+g0PCQwiEQYKCRwXEg33FzYoDPogQCKevgUOQQAAAABJRU5ErkJggg==",
    pixels:
      "083759801d4a59802e5486ad2a4d8baa395da2d0438faad5303b80813b466d943762a1a45978a8d95996abed70a5aedb435f90c13c6f9bbd688299c47d87c2d77caabee37bb0d8f66183acd558a0ccd876a2baf08b9ee0f58fbeddff9bdd071b8394ccf87da1db0189c3ec0793cc081ea5d9ff35db010b2f",
  },
  {
    name: "ecrit a la main en Python, 2 canaux, 6x5, filtre 2",
    channels: 2,
    width: 6,
    height: 5,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAYAAAAFCAQAAADMUVVtAAAASUlEQVR42mPiUpWzFveTj/PLjOtn4tX/oyeraCIgIskszST9z5Zbl41dTENOXoRJV0FCQE3H2ZZdyJqHSVFa2lpUX0TC0khGBACMrwq6gZmevwAAAABJRU5ErkJggg==",
    pixels:
      "0a251e3b174e1f5e4e695e8f17541a69346f536e628261aa3252577461755a848aa080be5f726f8487a19dc191b2bbca808d8abf9cd0b1d9cae4d7de",
  },
  {
    name: "ecrit a la main en Python, 4 canaux, 6x5, filtre 3",
    channels: 4,
    width: 6,
    height: 5,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAYAAAAFCAYAAABmWJ3mAAAAhUlEQVR42mP+o+Ze1KFg6sJvbx8lbuZUqGDh528ZmJHNPNvEJV1PU01aXkacS17GUFBDXk1UVkRCilnH3C9fmF9agl9RWI5HVEpNRIKZT/I/szqzf3R2jDAzn5CivIEYnyCLiLiC/HI+VUkJZovs2AUarKqqkrLM/TxculySHHVyRkoGfwBJ2Bf5lNGhwQAAAABJRU5ErkJggg==",
    pixels:
      "fc2647720633587d12586b98206277bd306989ad5185acc1194767a03d6685a9467b8faa528ab4c46998c4cd7aa2d0e1385a81bf4d6f9ecc5896a9d961a5c8f479b6c9ee92abcf0e6b88abbb6f7eb2d584a9dded80b8d60493d7ee20a0e6f72f6dafb2fd969bd70ea6bfdd0c9fc50612b2d6f837db00272f",
  },
  {
    name: "ecrit a la main en Python, 2 canaux, 6x5, filtre 4",
    channels: 2,
    width: 6,
    height: 5,
    png: "iVBORw0KGgoAAAANSUhEUgAAAAYAAAAFCAQAAADMUVVtAAAASklEQVR42mP5ZK3zk1FIiFVaQUicxeAvj4yoJK+QphLXJxY+S4kfDNwiOlxvGFVZFIRk5USEfn/SdRQRYVEXYBBX/MdkJM/EyQ4A1xsNhITMmKQAAAAASUVORK5CYII=",
    pixels:
      "f23b1e341f46314b4c6b5e8222382a503f694c7b759d7f8f30714869487460a77f9380b450836da181b37ca5acd4c0e8779377b8a2b6a4d7cbd9d4ef",
  },
];

function patterned(channels, width, height) {
  const pixels = new Uint8Array(width * height * channels);
  for (let i = 0; i < pixels.length; i++) pixels[i] = (i * 37 + (i % 7) * 11) % 256;
  return pixels;
}

describe("readPng", () => {
  it.each(REFERENCES)(
    "lit comme la reference externe : $name",
    ({ channels, width, height, png, pixels }) => {
      const decoded = readPng(Buffer.from(png, "base64"));
      expect([decoded.width, decoded.height, decoded.channels]).toEqual([width, height, channels]);
      expect(Buffer.from(decoded.pixels).toString("hex")).toBe(pixels);
    }
  );

  it.each([0, 1, 2, 3, 4])("relit ce que le filtre %i a ecrit", (filter) => {
    for (const channels of [1, 2, 3, 4]) {
      const pixels = patterned(channels, 13, 9);
      const decoded = readPng(encodePng({ width: 13, height: 9, channels, pixels, filter }));
      expect(Array.from(decoded.pixels)).toEqual(Array.from(pixels));
    }
  });

  it("rend le canal alpha d'une image a transparence, 255 sinon, 0 hors de l'image", () => {
    const la = readPng(
      encodePng({ width: 2, height: 1, channels: 2, pixels: Uint8Array.from([9, 0, 9, 200]) })
    );
    expect([alphaAt(la, 0, 0), alphaAt(la, 1, 0)]).toEqual([0, 200]);
    const rgba = readPng(
      encodePng({ width: 1, height: 1, channels: 4, pixels: Uint8Array.from([1, 2, 3, 77]) })
    );
    expect(alphaAt(rgba, 0, 0)).toBe(77);
    const rgb = readPng(
      encodePng({ width: 1, height: 1, channels: 3, pixels: Uint8Array.from([1, 2, 3]) })
    );
    expect(alphaAt(rgb, 0, 0)).toBe(255);
    expect([alphaAt(la, -1, 0), alphaAt(la, 2, 0), alphaAt(la, 0, 1)]).toEqual([0, 0, 0]);
  });

  it("refuse ce qui n'est pas un PNG ou n'est pas pris en charge", () => {
    expect(() => readPng(Buffer.from("pas un png du tout"))).toThrow(/signature/);
    const good = encodePng({ width: 2, height: 2, channels: 2, pixels: new Uint8Array(8) });
    expect(() => readPng(good.subarray(0, good.length - 20))).toThrow(
      /tronque|inattendue|incorrect/
    );
    const palette = Buffer.from(good);
    palette[8 + 8 + 9] = 3;
    expect(() => readPng(palette)).toThrow(/non pris en charge/);
    const interlaced = Buffer.from(good);
    interlaced[8 + 8 + 12] = 1;
    expect(() => readPng(interlaced)).toThrow(/non pris en charge/);
  });

  it("lit la figure livree : coins transparents, centre opaque", () => {
    for (const view of ["front", "back"]) {
      const png = readPng(readFileSync(`src/assets/body/${view}.png`));
      expect([png.width, png.height, png.channels]).toEqual([512, 1400, 2]);
      expect(alphaAt(png, 0, 0)).toBe(0);
      expect(alphaAt(png, 511, 1399)).toBe(0);
      expect(alphaAt(png, 256, 700)).toBe(255);
    }
  });
});
