import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ReconnectNoticeCard } from "./ReconnectNoticeCard";
import type { SourceInfo } from "@/features/sources/types";

function source(id: string, connected: boolean, credentials: boolean): SourceInfo {
  return {
    id,
    label: id === "oura" ? "Oura" : "Withings",
    kind: "oauth",
    extensions: [],
    domains: [],
    connected,
    credentials_configured: credentials,
    last_sync_at: null,
    redirect_uri: null,
    developer_url: null,
    reauth: null,
  };
}

const BOTH = {
  sources: [{ source: "withings", label: "Withings", tokens: true, credentials: true }],
};
const WITH_OURA = {
  sources: [
    { source: "withings", label: "Withings", tokens: true, credentials: false },
    { source: "oura", label: "Oura", tokens: true, credentials: true },
  ],
};

describe("ReconnectNoticeCard", () => {
  it("explique la situation et liste les étapes quand les secrets ont été invalidés", () => {
    render(<ReconnectNoticeCard secrets={BOTH} sources={[source("withings", false, false)]} />);

    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Reconnexion Withings nécessaire");
    expect(alert).toHaveTextContent(/mesures santé ne sont pas touchées/);
    expect(alert).toHaveTextContent(/Client ID/);
    expect(alert).toHaveTextContent(/Connecter/);
  });

  it("ne rend rien quand aucun secret n'a été invalidé", () => {
    const { container } = render(
      <ReconnectNoticeCard secrets={{ sources: [] }} sources={[source("withings", false, false)]} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("disparaît une fois les identifiants ressaisis et le compte reconnecté", () => {
    const { container } = render(
      <ReconnectNoticeCard secrets={BOTH} sources={[source("withings", true, true)]} />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it("suit l'état de chaque source : Oura garde son étape quand Withings est reconnecté", () => {
    render(
      <ReconnectNoticeCard
        secrets={WITH_OURA}
        sources={[source("withings", true, true), source("oura", false, true)]}
      />
    );

    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Reconnexion Withings, Oura nécessaire");
    expect(alert).toHaveTextContent(/Oura : cliquez sur Connecter/);
    expect(alert).not.toHaveTextContent(/Withings : cliquez sur Connecter/);
  });

  it("retire l'étape d'Oura dès qu'elle est reconnectée (N-62)", () => {
    const { container } = render(
      <ReconnectNoticeCard
        secrets={WITH_OURA}
        sources={[source("withings", true, true), source("oura", true, true)]}
      />
    );
    expect(container).toBeEmptyDOMElement();
  });
});
