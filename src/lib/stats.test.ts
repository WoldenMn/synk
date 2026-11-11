import { describe, it, expect } from "vitest";
import { avg, sum, latest } from "./stats";

type Record = { value: number | null };

describe("avg", () => {
  it("calcule la moyenne sur les valeurs non nulles", () => {
    const records: Record[] = [{ value: 10 }, { value: 20 }, { value: 30 }];
    expect(avg(records, "value")).toBe(20);
  });

  it("ignore les valeurs nulles dans le calcul", () => {
    const records: Record[] = [{ value: 10 }, { value: null }, { value: 30 }];
    expect(avg(records, "value")).toBe(20);
  });

  it("retourne null si toutes les valeurs sont nulles", () => {
    const records: Record[] = [{ value: null }, { value: null }];
    expect(avg(records, "value")).toBeNull();
  });

  it("retourne null pour tableau vide", () => {
    expect(avg<Record>([], "value")).toBeNull();
  });
});

describe("sum", () => {
  it("additionne les valeurs", () => {
    const records: Record[] = [{ value: 10 }, { value: 20 }, { value: 30 }];
    expect(sum(records, "value")).toBe(60);
  });

  it("traite null comme zero", () => {
    const records: Record[] = [{ value: 10 }, { value: null }, { value: 5 }];
    expect(sum(records, "value")).toBe(15);
  });

  it("retourne 0 pour tableau vide", () => {
    expect(sum<Record>([], "value")).toBe(0);
  });
});

describe("latest", () => {
  it("retourne la dernière valeur du tableau", () => {
    const records: Record[] = [{ value: 10 }, { value: 20 }, { value: 30 }];
    expect(latest(records, "value")).toBe(30);
  });

  it("retourne null pour tableau vide", () => {
    expect(latest<Record>([], "value")).toBeNull();
  });

  it("retourne null si la dernière valeur est null", () => {
    const records: Record[] = [{ value: 10 }, { value: null }];
    expect(latest(records, "value")).toBeNull();
  });
});
