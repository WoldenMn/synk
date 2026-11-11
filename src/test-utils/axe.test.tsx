import { afterEach, describe, expect, it } from "vitest";
import { expectNoAxeViolations } from "./axe";

function mount(html: string): HTMLElement {
  const container = document.createElement("div");
  container.innerHTML = html;
  document.body.append(container);
  return container;
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("expectNoAxeViolations", () => {
  it("échoue sur une image sans texte alternatif", async () => {
    const container = mount('<img src="x.png">');
    await expect(expectNoAxeViolations(container)).rejects.toThrow(/image-alt/);
  });

  it("passe sur un bouton nommé", async () => {
    const container = mount('<button type="button">Enregistrer</button>');
    await expectNoAxeViolations(container);
  });

  it("respecte les règles coupées par l'appelant", async () => {
    const container = mount('<img src="x.png">');
    await expectNoAxeViolations(container, { rules: { "image-alt": { enabled: false } } });
  });

  it("ne juge pas le contraste, que jsdom ne calcule pas", async () => {
    const container = mount('<p style="color:#fff;background:#fff">Texte</p>');
    await expectNoAxeViolations(container);
  });
});
