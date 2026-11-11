import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TimeRangeSelector } from "./TimeRangeSelector";

describe("TimeRangeSelector", () => {
  it("dit quelle période est active", () => {
    render(<TimeRangeSelector value="30d" onChange={vi.fn()} onCompareChange={vi.fn()} />);
    const pressed = screen.getAllByRole("button", { pressed: true });
    expect(pressed).toHaveLength(1);
    expect(pressed[0]).toHaveTextContent("30j");
    expect(screen.getByRole("button", { name: "Comparer" })).toHaveAttribute(
      "aria-pressed",
      "false"
    );
  });
});
