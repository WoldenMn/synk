import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "./dialog";

describe("Dialog a11y", () => {
  it("has role=dialog and aria-modal=true on content", () => {
    render(
      <Dialog open onOpenChange={vi.fn()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Titre</DialogTitle>
            <DialogDescription>Description</DialogDescription>
          </DialogHeader>
          <button>Action</button>
        </DialogContent>
      </Dialog>
    );
    const dialogs = screen.getAllByRole("dialog");
    expect(dialogs.length).toBeGreaterThan(0);
    const dialog = dialogs.find((d) => d.getAttribute("aria-modal") === "true");
    expect(dialog).toBeDefined();
  });

  it("links aria-labelledby to DialogTitle", () => {
    render(
      <Dialog open onOpenChange={vi.fn()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Mon titre</DialogTitle>
          </DialogHeader>
        </DialogContent>
      </Dialog>
    );
    const dialogs = screen.getAllByRole("dialog");
    const dialog = dialogs.find((d) => d.getAttribute("aria-modal") === "true");
    const labelledBy = dialog?.getAttribute("aria-labelledby");
    expect(labelledBy).toBeTruthy();
    const titleEl = document.getElementById(labelledBy!);
    expect(titleEl?.textContent).toBe("Mon titre");
  });

  it("traps focus within dialog on Tab", async () => {
    const user = userEvent.setup();
    render(
      <div>
        <button>Outside before</button>
        <Dialog open onOpenChange={vi.fn()}>
          <DialogContent>
            <button>First</button>
            <button>Middle</button>
            <button>Last</button>
          </DialogContent>
        </Dialog>
        <button>Outside after</button>
      </div>
    );
    const first = screen.getByText("First");
    const middle = screen.getByText("Middle");
    const last = screen.getByText("Last");

    first.focus();
    expect(first).toHaveFocus();
    await user.tab();
    expect(middle).toHaveFocus();
    await user.tab();
    expect(last).toHaveFocus();
    // Tab from last : should loop (not escape to Outside after)
    await user.tab();
    expect(screen.getByText("Outside after")).not.toHaveFocus();
  });
});
