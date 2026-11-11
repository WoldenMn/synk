import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useRef, type RefObject } from "react";
import { MemoryRouter, useNavigate } from "react-router-dom";
import { describe, expect, it } from "vitest";
import { PAGE_TITLES } from "@/lib/constants";
import { useRouteAnnouncement } from "./useRouteAnnouncement";

function Harness({ main }: { main: RefObject<HTMLElement | null> }) {
  const heading = useRef<HTMLHeadingElement>(null);
  useRouteAnnouncement(main, heading);
  const navigate = useNavigate();
  return (
    <>
      <h1 ref={heading} tabIndex={-1}>
        Titre
      </h1>
      <button type="button" onClick={() => navigate("/sleep")}>
        aller
      </button>
    </>
  );
}

describe("useRouteAnnouncement", () => {
  it("nomme la fenêtre, pose le focus sur le titre et remonte en haut à chaque changement de page", async () => {
    const main = { current: { scrollTop: 500 } as unknown as HTMLElement };
    render(
      <MemoryRouter initialEntries={["/"]}>
        <Harness main={main} />
      </MemoryRouter>
    );
    expect(document.title).toBe(`${PAGE_TITLES["/"]} | Synk`);
    expect(screen.getByRole("heading")).not.toHaveFocus();

    await userEvent.setup().click(screen.getByRole("button", { name: "aller" }));

    expect(document.title).toBe(`${PAGE_TITLES["/sleep"]} | Synk`);
    expect(screen.getByRole("heading")).toHaveFocus();
    expect(main.current.scrollTop).toBe(0);
  });
});
