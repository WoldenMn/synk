import { warningProblem } from "./parcours-lib.mjs";

export function connect(wsUrl, { WebSocketImpl = WebSocket, timeoutMs = 30000 } = {}) {
  const ws = new WebSocketImpl(wsUrl);
  let id = 0;
  const pending = new Map();
  const errors = [];
  const warnings = [];
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg);
      pending.delete(msg.id);
    } else if (msg.method === "Runtime.exceptionThrown") {
      errors.push(
        msg.params.exceptionDetails.exception?.description ?? msg.params.exceptionDetails.text
      );
    } else if (msg.method === "Runtime.consoleAPICalled" && msg.params.type === "error") {
      errors.push(msg.params.args.map((a) => a.value ?? a.description ?? "").join(" "));
    } else if (msg.method === "Runtime.consoleAPICalled" && msg.params.type === "warning") {
      const problem = warningProblem(
        msg.params.args.map((a) => a.value ?? a.description ?? "").join(" ")
      );
      if (problem) warnings.push(problem);
    } else if (msg.method === "Log.entryAdded" && msg.params.entry.level === "error") {
      errors.push(msg.params.entry.text);
    }
  };
  const ready = new Promise((resolve, reject) => {
    ws.onopen = resolve;
    ws.onerror = () => reject(new Error(`connexion CDP refusee : ${wsUrl}`));
  });
  const send = (method, params = {}) =>
    new Promise((resolve, reject) => {
      const n = ++id;
      const timer = setTimeout(() => {
        pending.delete(n);
        reject(new Error(`aucune reponse CDP a ${method} apres ${timeoutMs} ms`));
      }, timeoutMs);
      pending.set(n, (msg) => {
        clearTimeout(timer);
        resolve(msg);
      });
      ws.send(JSON.stringify({ id: n, method, params }));
    });
  const evaluate = async (expression) =>
    (await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true })).result
      ?.result?.value;
  return { ready, send, evaluate, errors, warnings, close: () => ws.close() };
}
