import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

vi.mock("@/features/sources/api", () => ({
  sourceSetCredentials: vi.fn(),
  listSources: vi.fn(() => Promise.resolve([])),
}));

import type { SourceInfo } from "@/features/sources/types";
import { SourceCredentials } from "./SourceCredentials";

const OURA: SourceInfo = {
  id: "oura",
  label: "Oura",
  kind: "oauth",
  extensions: [],
  domains: ["sleep", "activity", "workouts", "vitals"],
  connected: false,
  credentials_configured: false,
  last_sync_at: null,
  redirect_uri: "http://localhost:17483/callback/oura",
  developer_url: "https://cloud.ouraring.com/oauth/applications",
  reauth: null,
};

function renderCredentials(source: SourceInfo) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <SourceCredentials source={source} />
    </QueryClientProvider>
  );
}

describe("SourceCredentials", () => {
  it("guide la création de l'application Oura : lien, URL de rappel exacte, portées, limite", () => {
    renderCredentials(OURA);

    expect(screen.getByRole("link", { name: "cloud.ouraring.com" })).toHaveAttribute(
      "href",
      "https://cloud.ouraring.com/oauth/applications"
    );
    expect(screen.getByText("http://localhost:17483/callback/oura")).toBeInTheDocument();
    const steps = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
    expect(steps.some((s) => /My Applications/.test(s))).toBe(true);
    expect(steps.some((s) => /à l'identique/.test(s))).toBe(true);
    expect(steps.some((s) => /daily.*spo2.*workout/.test(s))).toBe(true);
    expect(steps.some((s) => /dix utilisateurs/.test(s))).toBe(true);
  });

  it("guide la création du client Polar : lien d'administration, URL de rappel exacte, consentements, fenêtre de 28 jours", () => {
    renderCredentials({
      ...OURA,
      id: "polar",
      label: "Polar",
      domains: ["sleep", "activity", "workouts", "vitals"],
      redirect_uri: "http://localhost:17483/callback/polar",
      developer_url: "https://admin.polaraccesslink.com/",
      reauth: null,
    });

    expect(screen.getByRole("link", { name: "admin.polaraccesslink.com" })).toHaveAttribute(
      "href",
      "https://admin.polaraccesslink.com/"
    );
    expect(screen.getByText("http://localhost:17483/callback/polar")).toBeInTheDocument();
    const steps = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
    expect(steps.some((s) => /Create client/.test(s))).toBe(true);
    expect(steps.some((s) => /à l'identique/.test(s))).toBe(true);
    expect(steps.some((s) => /consentements/.test(s))).toBe(true);
    expect(steps.some((s) => /28 jours/.test(s))).toBe(true);
  });

  it("guide la création de l'application Whoop : tableau de bord, portées, accès hors ligne, URL de rappel, limites", () => {
    renderCredentials({
      ...OURA,
      id: "whoop",
      label: "Whoop",
      domains: ["sleep", "activity", "workouts", "vitals"],
      redirect_uri: "http://localhost:17483/callback/whoop",
      developer_url: "https://developer-dashboard.whoop.com/",
      reauth: null,
    });

    expect(screen.getByRole("link", { name: "developer-dashboard.whoop.com" })).toHaveAttribute(
      "href",
      "https://developer-dashboard.whoop.com/"
    );
    expect(screen.getByText("http://localhost:17483/callback/whoop")).toBeInTheDocument();
    const steps = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
    expect(steps.some((s) => /read:recovery.*read:sleep.*read:cycles.*read:workout/.test(s))).toBe(
      true
    );
    expect(steps.some((s) => /offline/.test(s))).toBe(true);
    expect(steps.some((s) => /à l'identique/.test(s))).toBe(true);
    expect(steps.some((s) => /10 000 par jour/.test(s))).toBe(true);
  });

  it("guide la création du projet Google Health : API, utilisateur de test, portées, client Web, URL de rappel, jeton de 7 jours", () => {
    renderCredentials({
      ...OURA,
      id: "google_health",
      label: "Google Health",
      domains: ["sleep", "activity", "workouts", "vitals"],
      redirect_uri: "http://localhost:17483/callback/google_health",
      developer_url: "https://console.developers.google.com/apis/library/health.googleapis.com",
      reauth: null,
    });

    expect(screen.getByRole("link", { name: "console.developers.google.com" })).toHaveAttribute(
      "href",
      "https://console.developers.google.com/apis/library/health.googleapis.com"
    );
    expect(screen.getByText("http://localhost:17483/callback/google_health")).toBeInTheDocument();
    const steps = screen.getAllByRole("listitem").map((li) => li.textContent ?? "");
    expect(steps.some((s) => /Google Health API/.test(s) && /activez/.test(s))).toBe(true);
    expect(steps.some((s) => /utilisateur de test/.test(s))).toBe(true);
    expect(steps.some((s) => /sleep\.readonly/.test(s) && /health_metrics/.test(s))).toBe(true);
    expect(steps.some((s) => /Application Web/.test(s) && /à l'identique/.test(s))).toBe(true);
    expect(steps.some((s) => /application mobile Google Health/.test(s))).toBe(true);
    expect(steps.some((s) => /7 jours/.test(s))).toBe(true);
  });

  it("n'ajoute aucune liste d'étapes pour une marque sans aide propre", () => {
    renderCredentials({
      ...OURA,
      id: "withings",
      label: "Withings",
      redirect_uri: "http://localhost:17483/callback",
      developer_url: "https://developer.withings.com/",
      reauth: null,
    });

    expect(screen.queryAllByRole("listitem")).toHaveLength(0);
  });
});
