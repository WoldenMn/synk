import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  getAlerts,
  addAlert,
  deleteAlert,
  getAlertEvents,
  markAlertsSeen,
  checkAlerts,
} from "./api";

const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("alerts/api", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("getAlerts invoke get_alerts", async () => {
    const fake = [{ id: 1, metric_key: "weight", threshold: 80, direction: "above" }];
    mockInvoke.mockResolvedValue(fake);

    const result = await getAlerts();

    expect(mockInvoke).toHaveBeenCalledWith("get_alerts");
    expect(result).toEqual(fake);
  });

  it("addAlert invoke add_alert avec data wrappee", async () => {
    mockInvoke.mockResolvedValue(5);
    const data = {
      metric_key: "weight",
      condition: "above",
      threshold: 75,
      label: "Poids > 75 kg",
    };

    const result = await addAlert(data);

    expect(mockInvoke).toHaveBeenCalledWith("add_alert", { data });
    expect(result).toBe(5);
  });

  it("deleteAlert invoke delete_alert avec id", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await deleteAlert(11);

    expect(mockInvoke).toHaveBeenCalledWith("delete_alert", { id: 11 });
  });

  it("getAlertEvents invoke get_alert_events", async () => {
    const fake = [{ id: 1, alert_id: 5, triggered_at: "2026-04-25T08:00:00Z", value: 78 }];
    mockInvoke.mockResolvedValue(fake);

    const result = await getAlertEvents();

    expect(mockInvoke).toHaveBeenCalledWith("get_alert_events");
    expect(result).toEqual(fake);
  });

  it("markAlertsSeen invoke mark_alerts_seen", async () => {
    mockInvoke.mockResolvedValue(undefined);

    await markAlertsSeen();

    expect(mockInvoke).toHaveBeenCalledWith("mark_alerts_seen");
  });

  it("checkAlerts invoke evaluate_alerts et retourne nouveaux events", async () => {
    const fake = [{ id: 2, alert_id: 5, triggered_at: "2026-04-29T08:00:00Z", value: 81 }];
    mockInvoke.mockResolvedValue(fake);

    const result = await checkAlerts();

    expect(mockInvoke).toHaveBeenCalledWith("evaluate_alerts");
    expect(result).toEqual(fake);
  });
});
