import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import { DevicesCard } from "./DevicesCard";
import type { WithingsDevice } from "@/features/devices/types";

const DEVICE: WithingsDevice = {
  id: 1,
  device_id: "dev-1",
  device_type: "Scale",
  model: "Body Cardio",
  model_id: 6,
  battery: "high",
  last_session_date: null,
  first_session_date: null,
};

vi.mock("@/features/devices/hooks", () => ({
  useDevices: () => ({ data: [DEVICE] }),
}));

describe("DevicesCard", () => {
  it("affiche l'appareil et son badge de source sans bloc dans un paragraphe", () => {
    const { container } = render(<DevicesCard visible />);

    expect(container).toHaveTextContent("Body Cardio");
    expect(container).toHaveTextContent("Withings");
    expect(container.querySelector("p div")).toBeNull();
  });
});
