import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import type { OAuthStatus } from "@/features/sources/useOAuthFlow";

const flow = vi.hoisted(() => ({
  oauthStatus: "idle" as OAuthStatus,
  oauthError: null as string | null,
  handleConnect: vi.fn(),
  handleCancelConnect: vi.fn(),
}));
vi.mock("@/features/sources/useOAuthFlow", () => ({ useOAuthFlow: () => flow }));
vi.mock("@/features/sources/components/CredentialsForm", () => ({ CredentialsForm: () => null }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

import type { SourceInfo } from "@/features/sources/types";
import { SourceConnect } from "./SourceConnect";

const OURA: SourceInfo = {
  id: "oura",
  label: "Oura",
  kind: "oauth",
  extensions: [],
  domains: ["sleep"],
  connected: false,
  credentials_configured: true,
  last_sync_at: null,
  redirect_uri: null,
  developer_url: null,
  reauth: null,
};

function show(status: OAuthStatus, error: string | null = null) {
  flow.oauthStatus = status;
  flow.oauthError = error;
  render(<SourceConnect source={OURA} onConnected={vi.fn()} onBack={vi.fn()} />);
}

describe("SourceConnect", () => {
  it("annonce l'attente de l'autorisation comme un état", () => {
    show("waiting");
    expect(screen.getByRole("status")).toHaveTextContent("En attente de l'autorisation Oura");
  });

  it("annonce l'erreur de connexion comme une alerte", () => {
    show("error", "Accès refusé par Oura");
    expect(screen.getByRole("alert")).toHaveTextContent("Accès refusé par Oura");
  });
});
