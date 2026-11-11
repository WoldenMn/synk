export type RustParam = { name: string; optional: boolean };

export function splitTopLevel(text: string, quotes = "\"'`"): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  let quote: string | null = null;
  for (const ch of text) {
    if (quote) {
      current += ch;
      if (ch === quote) quote = null;
      continue;
    }
    if (quotes.includes(ch)) quote = ch;
    if ("([{<".includes(ch)) depth++;
    if (")]}>".includes(ch)) depth--;
    if (ch === "," && depth === 0) {
      parts.push(current.trim());
      current = "";
    } else {
      current += ch;
    }
  }
  if (current.trim()) parts.push(current.trim());
  return parts;
}

export function closing(text: string, start: number, open: string, close: string): number {
  let depth = 0;
  for (let i = start; i < text.length; i++) {
    if (text[i] === open) depth++;
    if (text[i] === close && --depth === 0) return i;
  }
  return -1;
}

export function registeredCommands(lib: string): string[] {
  const block = lib.match(/generate_handler!\[([\s\S]*?)\]\)/)?.[1] ?? "";
  return block
    .split("\n")
    .map((l) => l.replace(/\/\/.*$/, ""))
    .join("\n")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean)
    .map((s) => s.split("::").pop() as string);
}

export function rustParams(sources: Record<string, string>): Map<string, RustParam[]> {
  const out = new Map<string, RustParam[]>();
  for (const src of Object.values(sources)) {
    for (const m of src.matchAll(
      /#\[tauri::command\]\s*pub(?:\([a-z]+\))?\s+(?:async\s+)?fn\s+([a-z0-9_]+)\s*(?:<[^>]*>)?\s*\(/g
    )) {
      const open = (m.index ?? 0) + m[0].length - 1;
      const params = splitTopLevel(src.slice(open + 1, closing(src, open, "(", ")")), '"')
        .map((p) => p.split(":"))
        .filter((p) => p.length >= 2)
        .map(([name, ...type]) => ({ name: (name ?? "").trim(), type: type.join(":").trim() }))
        .filter((p) => !/^(tauri::)?(State|AppHandle|Window|WebviewWindow|Webview)\b/.test(p.type))
        .map((p) => ({ name: p.name, optional: p.type.startsWith("Option<") }));
      out.set(m[1] ?? "", params);
    }
  }
  return out;
}

export function commandFiles(sources: Record<string, string>): Map<string, string> {
  const files = new Map<string, string>();
  for (const [path, text] of Object.entries(sources)) {
    for (const m of text.matchAll(
      /#\[tauri::command[^\]]*\]\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)/g
    )) {
      files.set(m[1] ?? "", path.replace(/^.*src-tauri\//, "src-tauri/"));
    }
  }
  return files;
}

const camel = (s: string) => s.replace(/_([a-z0-9])/g, (_, c: string) => c.toUpperCase());

export function renderCatalog(lib: string, sources: Record<string, string>): string {
  const params = rustParams(sources);
  const files = commandFiles(sources);
  const rows = registeredCommands(lib)
    .sort()
    .map((name) => {
      const args =
        (params.get(name) ?? []).map((p) => camel(p.name) + (p.optional ? "?" : "")).join(", ") ||
        "aucun";
      return `| \`${name}\` | ${args} | \`${files.get(name) ?? "introuvable"}\` |`;
    });
  return [
    "# Commandes IPC",
    "",
    "Fichier généré par `npm run doc:commandes` depuis `src-tauri/src/lib.rs` et les signatures Rust : ne pas l'éditer à la main. Le gardien `src/__tests__/doc-drift.test.ts` le compare au code à chaque `npm run verify`.",
    "",
    `${rows.length} commandes.`,
    "",
    "| Commande | Arguments (côté front, en camelCase ; `?` = facultatif) | Fichier |",
    "|---|---|---|",
    ...rows,
    "",
  ].join("\n");
}
