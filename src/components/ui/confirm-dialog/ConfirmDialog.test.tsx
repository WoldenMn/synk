import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expectNoAxeViolations } from "@/test-utils/axe";
import { useState } from "react";
import { ConfirmProvider, useConfirm } from "./index";
import type { ConfirmOptions } from "./useConfirm";

function Harness({
  options,
  onResult,
}: {
  options: ConfirmOptions;
  onResult?: (result: boolean) => void;
}) {
  const confirm = useConfirm();
  const [result, setResult] = useState<string>("none");
  return (
    <div>
      <button
        onClick={async () => {
          const r = await confirm(options);
          setResult(String(r));
          onResult?.(r);
        }}
      >
        trigger
      </button>
      <span data-testid="result">{result}</span>
    </div>
  );
}

function renderWithProvider(ui: React.ReactNode) {
  return render(<ConfirmProvider>{ui}</ConfirmProvider>);
}

describe("useConfirm", () => {
  it("throws an explicit error when used outside a ConfirmProvider", () => {
    function Bare() {
      useConfirm();
      return null;
    }
    const spy = vi.spyOn(console, "error").mockImplementation(() => undefined);
    expect(() => render(<Bare />)).toThrow(/ConfirmProvider/);
    spy.mockRestore();
  });
});

describe("ConfirmDialog", () => {
  it("opens the dialog with title/description and resolves true on Confirmer", async () => {
    const user = userEvent.setup();
    renderWithProvider(
      <Harness
        options={{
          title: "Supprimer cette pesée ?",
          description: "Cette action est irreversible.",
        }}
      />
    );

    expect(screen.queryByRole("dialog")).toBeNull();

    await user.click(screen.getByText("trigger"));

    expect(screen.getByText("Supprimer cette pesée ?")).toBeInTheDocument();
    expect(screen.getByText("Cette action est irreversible.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Confirmer" }));

    await waitFor(() => {
      expect(screen.getByTestId("result")).toHaveTextContent("true");
    });
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("resolves false on Annuler", async () => {
    const user = userEvent.setup();
    renderWithProvider(<Harness options={{ title: "Confirmer ?" }} />);

    await user.click(screen.getByText("trigger"));
    await user.click(screen.getByRole("button", { name: "Annuler" }));

    await waitFor(() => {
      expect(screen.getByTestId("result")).toHaveTextContent("false");
    });
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("resolves false on Escape", async () => {
    const user = userEvent.setup();
    renderWithProvider(<Harness options={{ title: "Confirmer ?" }} />);

    await user.click(screen.getByText("trigger"));
    expect(screen.getByText("Confirmer ?")).toBeInTheDocument();

    await user.keyboard("{Escape}");

    await waitFor(() => {
      expect(screen.getByTestId("result")).toHaveTextContent("false");
    });
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("uses the destructive button variant when destructive is true", async () => {
    const user = userEvent.setup();
    renderWithProvider(<Harness options={{ title: "Supprimer ?", destructive: true }} />);

    await user.click(screen.getByText("trigger"));

    const confirmBtn = screen.getByRole("button", { name: "Confirmer" });
    expect(confirmBtn.className).toMatch(/bg-destructive/);
  });

  it("uses a non-destructive variant when destructive is false", async () => {
    const user = userEvent.setup();
    renderWithProvider(<Harness options={{ title: "Continuer ?", destructive: false }} />);

    await user.click(screen.getByText("trigger"));

    const confirmBtn = screen.getByRole("button", { name: "Confirmer" });
    expect(confirmBtn.className).not.toMatch(/bg-destructive/);
  });

  it("supports custom confirm/cancel labels", async () => {
    const user = userEvent.setup();
    renderWithProvider(
      <Harness
        options={{
          title: "Vider le cache ?",
          confirmLabel: "Vider",
          cancelLabel: "Garder",
        }}
      />
    );

    await user.click(screen.getByText("trigger"));

    expect(screen.getByRole("button", { name: "Vider" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Garder" })).toBeInTheDocument();
  });

  it("has no a11y violations (role=dialog, aria-modal, linked title)", async () => {
    const user = userEvent.setup();
    const { container } = renderWithProvider(
      <Harness
        options={{
          title: "Supprimer cet élément ?",
          description: "Confirme la suppression.",
        }}
      />
    );

    await user.click(screen.getByText("trigger"));

    const dialogs = screen.getAllByRole("dialog");
    const dialog = dialogs.find((d) => d.getAttribute("aria-modal") === "true");
    expect(dialog).toBeDefined();
    const labelledBy = dialog?.getAttribute("aria-labelledby");
    expect(labelledBy).toBeTruthy();
    expect(document.getElementById(labelledBy!)?.textContent).toBe("Supprimer cet élément ?");

    await expectNoAxeViolations(container, {
      rules: { "heading-order": { enabled: false } },
    });
  });
});
