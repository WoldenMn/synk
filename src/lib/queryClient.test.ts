import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Query } from "@tanstack/react-query";
import { createQueryClient, createToastThrottle } from "./queryClient";
import { reportError } from "./reportError";

vi.mock("./reportError", () => ({ reportError: vi.fn() }));

describe("createToastThrottle", () => {
  it("laisse passer un toast par clé et par minute", () => {
    const allow = createToastThrottle(60_000);
    expect(allow('["sleep"]', 0)).toBe(true);
    expect(allow('["sleep"]', 30_000)).toBe(false);
    expect(allow('["activity"]', 30_000)).toBe(true);
    expect(allow('["sleep"]', 60_001)).toBe(true);
  });
});

describe("createQueryClient", () => {
  it("invalide les cartes du tableau de bord apres toute ecriture reussie", async () => {
    const client = createQueryClient();
    client.setQueryData(["dashboardCards", "a", "b"], { body: null });
    client.setQueryData(["goals"], []);

    await client
      .getMutationCache()
      .build(client, { mutationFn: () => Promise.resolve(1) })
      .execute(undefined);

    expect(client.getQueryState(["dashboardCards", "a", "b"])?.isInvalidated).toBe(true);
    expect(client.getQueryState(["goals"])?.isInvalidated).toBe(false);
  });

  it("laisse les cartes en cache apres une ecriture en echec", async () => {
    const client = createQueryClient();
    client.setQueryData(["dashboardCards", "a", "b"], { body: null });

    await client
      .getMutationCache()
      .build(client, { mutationFn: () => Promise.reject(new Error("refus")) })
      .execute(undefined)
      .catch(() => undefined);

    expect(client.getQueryState(["dashboardCards", "a", "b"])?.isInvalidated).toBe(false);
  });
});

describe("erreur d'une requete", () => {
  const fail = (data: unknown, meta?: Record<string, unknown>) => {
    const query = { state: { data }, meta, queryKey: ["k"] } as unknown as Query<unknown, unknown>;
    createQueryClient().getQueryCache().config.onError?.(new Error("x"), query);
  };

  beforeEach(() => {
    vi.mocked(reportError).mockClear();
  });

  it("ne signale rien sans donnee a l'ecran : la page porte l'erreur", () => {
    fail(undefined);
    expect(reportError).not.toHaveBeenCalled();
  });

  it("signale quand une donnee est deja a l'ecran", () => {
    fail([1]);
    expect(reportError).toHaveBeenCalledTimes(1);
  });

  it("signale toujours pour une requete du bandeau", () => {
    fail(undefined, { surface: "shell" });
    expect(reportError).toHaveBeenCalledTimes(1);
  });
});
