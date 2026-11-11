import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useKeyboardShortcuts, type Shortcut } from "./useKeyboardShortcuts";
import { SHORTCUTS } from "@/lib/shortcuts";

function Harness({ shortcuts }: { shortcuts: Shortcut[] }) {
  useKeyboardShortcuts(shortcuts);
  return <input aria-label="champ" />;
}

describe("useKeyboardShortcuts", () => {
  it("lance l'action du raccourci", () => {
    const run = vi.fn();
    render(<Harness shortcuts={[{ key: "s", ctrl: true, shift: true, run }]} />);
    fireEvent.keyDown(window, { key: "S", ctrlKey: true, shiftKey: true });
    expect(run).toHaveBeenCalledTimes(1);
  });

  it("ignore la touche seule et la saisie dans un champ", () => {
    const run = vi.fn();
    const { getByLabelText } = render(<Harness shortcuts={[{ key: "b", ctrl: true, run }]} />);
    fireEvent.keyDown(window, { key: "b" });
    fireEvent.keyDown(getByLabelText("champ"), { key: "b", ctrlKey: true });
    expect(run).not.toHaveBeenCalled();
  });

  it("exige la bonne combinaison de modificateurs", () => {
    const run = vi.fn();
    render(<Harness shortcuts={[{ key: "b", ctrl: true, run }]} />);
    fireEvent.keyDown(window, { key: "b", ctrlKey: true, shiftKey: true });
    fireEvent.keyDown(window, { key: "b", ctrlKey: true, altKey: true });
    expect(run).not.toHaveBeenCalled();
    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(run).toHaveBeenCalledTimes(1);
  });

  it("retient l'événement pour que le navigateur n'en fasse rien", () => {
    const run = vi.fn();
    render(<Harness shortcuts={[{ key: "s", ctrl: true, shift: true, run }]} />);
    const event = new KeyboardEvent("keydown", {
      key: "S",
      ctrlKey: true,
      shiftKey: true,
      cancelable: true,
    });
    window.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  it("se désabonne au démontage", () => {
    const run = vi.fn();
    const { unmount } = render(<Harness shortcuts={[{ key: "b", ctrl: true, run }]} />);
    unmount();
    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(run).not.toHaveBeenCalled();
  });

  it("ignore les trois raccourcis tant qu'un dialogue modal est ouvert", () => {
    const runs = Object.values(SHORTCUTS).map(() => vi.fn());
    const shortcuts = Object.values(SHORTCUTS).map((d, i) => ({
      key: d.key,
      ctrl: d.ctrl,
      shift: d.shift,
      run: runs[i] as () => void,
    }));
    const { unmount } = render(
      <>
        <Harness shortcuts={shortcuts} />
        <div role="dialog" aria-modal="true" aria-label="Boîte" />
      </>
    );
    for (const d of Object.values(SHORTCUTS)) {
      fireEvent.keyDown(window, { key: d.key.toUpperCase(), ctrlKey: d.ctrl, shiftKey: d.shift });
    }
    for (const run of runs) expect(run).not.toHaveBeenCalled();
    unmount();
    render(<Harness shortcuts={shortcuts} />);
    for (const d of Object.values(SHORTCUTS)) {
      fireEvent.keyDown(window, { key: d.key.toUpperCase(), ctrlKey: d.ctrl, shiftKey: d.shift });
    }
    for (const run of runs) expect(run).toHaveBeenCalledTimes(1);
  });

  it("n'est pas arrêté par un dialogue non modal", () => {
    const run = vi.fn();
    render(
      <>
        <Harness shortcuts={[{ key: "b", ctrl: true, run }]} />
        <div role="dialog" aria-label="Info" />
      </>
    );
    fireEvent.keyDown(window, { key: "b", ctrlKey: true });
    expect(run).toHaveBeenCalledTimes(1);
  });
});
