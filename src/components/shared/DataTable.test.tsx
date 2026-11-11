import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DataTable } from "./DataTable";

const rows = [
  { id: 1, label: "manuelle", source: "manual" },
  { id: 2, label: "withings", source: "withings" },
];

describe("DataTable", () => {
  it("n'offre la corbeille qu'aux lignes autorisées", () => {
    render(
      <DataTable
        label="Essais"
        columns={[{ key: "label", label: "Libellé" }]}
        data={rows}
        onDelete={vi.fn()}
        rowLabel={(r) => `la ligne ${r.label}`}
        canDelete={(r) => r.source === "manual"}
        emptyMessage="vide"
      />
    );
    expect(screen.getAllByRole("button", { name: /Supprimer/ })).toHaveLength(1);
  });

  it("pagine à 50 lignes", () => {
    const many = Array.from({ length: 120 }, (_, i) => ({ id: i + 1, label: `l${i}` }));
    render(
      <DataTable
        label="Essais"
        columns={[{ key: "label", label: "Libellé" }]}
        data={many}
        emptyMessage="vide"
      />
    );
    expect(screen.getAllByRole("row")).toHaveLength(51);
  });

  it("nomme le tableau, la recherche et chaque suppression", () => {
    render(
      <DataTable
        label="Pesées"
        columns={[{ key: "value", label: "Poids" }]}
        data={[
          { id: 1, value: 72 },
          { id: 2, value: 71 },
        ]}
        onDelete={vi.fn()}
        rowLabel={(row) => `la pesée de ${row.value} kg`}
        emptyMessage="Aucune pesée"
      />
    );
    expect(screen.getByRole("table", { name: "Pesées" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Rechercher dans Pesées" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Supprimer la pesée de 72 kg" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Supprimer la pesée de 71 kg" })).toBeInTheDocument();
  });

  it("cherche dans le texte de chaque colonne", async () => {
    const user = userEvent.setup();
    render(
      <DataTable
        label="Pesées"
        columns={[
          { key: "date", label: "Date", render: (r) => r.label, searchText: (r) => r.label },
          {
            key: "value",
            label: "Poids",
            render: (r) => `${r.value} kg`,
            searchText: (r) => `${r.value} kg`,
          },
        ]}
        data={[
          { id: 1, label: "28 juin 2026", value: 76 },
          { id: 2, label: "3 juillet 2026", value: 75 },
        ]}
        emptyMessage="Aucune pesée"
      />
    );
    const field = screen.getByRole("textbox", { name: "Rechercher dans Pesées" });
    await user.type(field, "juin");
    expect(screen.getByText("28 juin 2026")).toBeInTheDocument();
    expect(screen.queryByText("3 juillet 2026")).not.toBeInTheDocument();
    await user.clear(field);
    await user.type(field, "75");
    expect(screen.getByText("3 juillet 2026")).toBeInTheDocument();
    expect(screen.queryByText("28 juin 2026")).not.toBeInTheDocument();
  });

  it("ne cherche pas dans une colonne rendue sans texte de recherche", async () => {
    const user = userEvent.setup();
    render(
      <DataTable
        label="Essais"
        columns={[{ key: "label", label: "Libellé", render: (r) => r.label }]}
        data={[{ id: 1, label: "alpha" }]}
        emptyMessage="vide"
      />
    );
    await user.type(screen.getByRole("textbox", { name: "Rechercher dans Essais" }), "alpha");
    expect(screen.getByText("Aucun résultat")).toBeInTheDocument();
  });
});
