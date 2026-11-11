import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import type { ProfileCard } from "@/features/profileCard/types";

const emptyCard: ProfileCard = {
  body: {
    age: null,
    sex: null,
    height_cm: null,
    weight_kg: null,
    bmi: null,
    fat_ratio: null,
    muscle_mass: null,
    hydration: null,
    measured_at: null,
  },
  key_measurements: [],
  health: { active_treatments: [] },
  sizes: [],
  records: [],
};

const fullCard: ProfileCard = {
  body: {
    age: 36,
    sex: "male",
    height_cm: 178,
    weight_kg: 74.5,
    bmi: 23.5,
    fat_ratio: 18.2,
    muscle_mass: 58.1,
    hydration: 55.4,
    measured_at: "2026-08-01",
  },
  key_measurements: [
    { slug: "waist", label: "Tour de taille", value: 86, measured_at: "2026-07-31" },
  ],
  health: { active_treatments: ["Doliprane"] },
  sizes: [
    { category: "top", value: "L" },
    { category: "shoes", value: "43" },
  ],
  records: [
    {
      key: "weight_min",
      label: "Poids minimum",
      value: 74.5,
      unit: "kg",
      achieved_at: "2026-04-20",
    },
  ],
};

let currentCard: ProfileCard = fullCard;

vi.mock("@/features/profileCard/hooks", () => ({
  useProfileCard: () => ({ data: currentCard, isLoading: false }),
  useExportProfileCardPdf: () => ({ mutate: vi.fn(), isPending: false }),
}));

import Profile from "./Profile";

function mount() {
  return render(
    <MemoryRouter>
      <Profile />
    </MemoryRouter>
  );
}

describe("Page Profil", () => {
  it("affiche les cinq blocs", () => {
    currentCard = fullCard;
    mount();
    for (const title of ["Corps", "Mensurations", "Traitements en cours", "Tailles", "Records"]) {
      expect(screen.getByRole("heading", { name: new RegExp(title, "i") })).toBeInTheDocument();
    }
  });

  it("affiche les valeurs connues", () => {
    currentCard = fullCard;
    mount();
    expect(screen.getByText("36 ans")).toBeInTheDocument();
    expect(screen.getByText("Doliprane")).toBeInTheDocument();
  });

  it("traduit la catégorie shoes en Pointure", () => {
    currentCard = fullCard;
    mount();
    expect(screen.getByText("Pointure")).toBeInTheDocument();
    expect(screen.queryByText("shoes")).not.toBeInTheDocument();
  });

  it("ne plante pas sur une base vide et affiche des tirets", () => {
    currentCard = emptyCard;
    mount();
    expect(screen.getByText(/Corps/i)).toBeInTheDocument();
    expect(screen.getAllByText("—").length).toBeGreaterThan(0);
  });

  it("propose l'export", () => {
    currentCard = fullCard;
    mount();
    expect(screen.getByRole("button", { name: /exporter/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /modifier/i })).not.toBeInTheDocument();
  });

  it("range les blocs en deux colonnes indépendantes", () => {
    currentCard = fullCard;
    mount();
    const parentOf = (title: string) =>
      screen.getByRole("heading", { name: new RegExp(title, "i") }).closest("section")
        ?.parentElement;
    expect(parentOf("Corps")).toBe(parentOf("Traitements en cours"));
    expect(parentOf("Mensurations")).toBe(parentOf("Tailles"));
    expect(parentOf("Corps")).not.toBe(parentOf("Mensurations"));
    expect(parentOf("Corps")?.className).toContain("flex-col");
  });

  it("renvoie vers Paramètres quand âge, sexe ou taille manquent", () => {
    currentCard = emptyCard;
    mount();
    expect(screen.getByRole("link", { name: "Renseigner dans Paramètres" })).toHaveAttribute(
      "href",
      "/settings"
    );
  });

  it("n'offre pas ce lien quand le profil est complet", () => {
    currentCard = fullCard;
    mount();
    expect(screen.queryByRole("link", { name: "Renseigner dans Paramètres" })).toBeNull();
  });
});
