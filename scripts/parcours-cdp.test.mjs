import { describe, expect, it } from "vitest";
import { connect } from "./parcours-cdp.mjs";

class SilentSocket {
  constructor() {
    queueMicrotask(() => this.onopen?.());
  }
  send() {}
  close() {}
}

class AnsweringSocket extends SilentSocket {
  send(raw) {
    const { id } = JSON.parse(raw);
    queueMicrotask(() =>
      this.onmessage({ data: JSON.stringify({ id, result: { result: { value: "ok" } } }) })
    );
  }
}

describe("connect", () => {
  it("rend la valeur d'une page qui repond (cas connu bon)", async () => {
    const cdp = connect("ws://fake", { WebSocketImpl: AnsweringSocket, timeoutMs: 200 });
    await cdp.ready;
    expect(await cdp.evaluate("1")).toBe("ok");
  });

  it("n'attend pas sans fin une page qui ne repond pas (cas connu mauvais)", async () => {
    const cdp = connect("ws://fake", { WebSocketImpl: SilentSocket, timeoutMs: 100 });
    await cdp.ready;
    await expect(cdp.evaluate("1")).rejects.toThrow(/aucune reponse/);
  }, 2000);
});

describe("connect, avertissements", () => {
  class WarningSocket extends SilentSocket {
    constructor() {
      super();
      queueMicrotask(() => {
        for (const value of ["IPC custom protocol failed, repli", "avertissement ordinaire"]) {
          this.onmessage({
            data: JSON.stringify({
              method: "Runtime.consoleAPICalled",
              params: { type: "warning", args: [{ value }] },
            }),
          });
        }
      });
    }
  }

  it("garde le repli de l'IPC et ignore le reste", async () => {
    const cdp = connect("ws://fake", { WebSocketImpl: WarningSocket, timeoutMs: 100 });
    await cdp.ready;
    await new Promise((r) => setTimeout(r, 0));
    expect(cdp.warnings).toHaveLength(1);
    expect(cdp.warnings[0]).toMatch(/postMessage/);
  });
});
