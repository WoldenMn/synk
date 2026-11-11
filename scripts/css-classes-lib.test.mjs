import { describe, expect, it } from "vitest";
import { cssClassNames } from "./css-classes-lib.mjs";

describe("cssClassNames", () => {
  it("rend les classes sans échappement ni pseudo-classe", () => {
    const css =
      ".hover\\:glow-rose:hover{box-shadow:none}.bg-card\\/80{color:red}.dark .glass{x:y}@media (hover:hover){.hover\\:bg-muted:hover{a:b}}";
    expect(cssClassNames(css)).toEqual([
      "bg-card/80",
      "dark",
      "glass",
      "hover:bg-muted",
      "hover:glow-rose",
    ]);
  });

  it("ignore les nombres décimaux des valeurs", () => {
    expect(cssClassNames(".p-1{padding:.25rem;opacity:0.5}")).toEqual(["p-1"]);
  });
});
