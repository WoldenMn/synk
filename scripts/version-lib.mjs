const SEMVER = /^\d+\.\d+\.\d+$/;

export function isSemver(version) {
  return SEMVER.test(version);
}

const PACKAGE_LOCK_ROOT = /("packages":\s*\{\s*"":\s*\{[^}]*?"version":\s*")[^"]+(")/;
const PACKAGE_LOCK_TOP = /^(\s{2}"version":\s*")[^"]+(")/m;
const JSON_VERSION = /("version":\s*")[^"]+(")/;
const CARGO_TOML = /^(\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m;
const CARGO_LOCK = /(name = "synk"\nversion = ")[^"]+(")/;
const README = /^(Version courante : )\d+\.\d+\.\d+()/m;

function json(text) {
  return JSON.parse(text);
}

const READERS = {
  "package.json": (t) => ({ "package.json": json(t).version }),
  "package-lock.json": (t) => ({
    "package-lock.json": json(t).version,
    "package-lock.json#racine": json(t).packages?.[""]?.version,
  }),
  "src-tauri/Cargo.toml": (t) => ({
    "Cargo.toml": t.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m)?.[1],
  }),
  "src-tauri/Cargo.lock": (t) => ({
    "Cargo.lock": t.match(/name = "synk"\nversion = "([^"]+)"/)?.[1],
  }),
  "src-tauri/tauri.conf.json": (t) => ({ "tauri.conf.json": json(t).version }),
  "README.md": (t) => ({ "README.md": t.match(/^Version courante : (\d+\.\d+\.\d+)/m)?.[1] }),
};

export const VERSION_FILES = Object.keys(READERS);

function normalize(text) {
  return text.replace(/\r\n/g, "\n");
}

export function readVersions(read) {
  return Object.assign({}, ...VERSION_FILES.map((p) => READERS[p](normalize(read(p)))));
}

export function disagreements(versions) {
  const known = Object.values(versions).filter((v) => v !== undefined);
  const counts = new Map();
  for (const v of known) counts.set(v, (counts.get(v) ?? 0) + 1);
  const majority = [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0];
  return Object.fromEntries(Object.entries(versions).filter(([, v]) => v !== majority));
}

function replaceOnce(path, text, pattern, version) {
  if (!pattern.test(text)) throw new Error(`emplacement de version introuvable dans ${path}`);
  return text.replace(pattern, `$1${version}$2`);
}

export function bumpText(path, text, version) {
  if (!isSemver(version)) throw new Error(`version mal formee : ${version}`);
  switch (path) {
    case "package.json":
    case "src-tauri/tauri.conf.json":
      return replaceOnce(path, text, JSON_VERSION, version);
    case "package-lock.json":
      return replaceOnce(
        path,
        replaceOnce(path, text, PACKAGE_LOCK_TOP, version),
        PACKAGE_LOCK_ROOT,
        version
      );
    case "src-tauri/Cargo.toml":
      return replaceOnce(path, text, CARGO_TOML, version);
    case "src-tauri/Cargo.lock":
      return replaceOnce(path, text, CARGO_LOCK, version);
    case "README.md":
      return replaceOnce(path, text, README, version);
    default:
      throw new Error(`fichier de version inconnu : ${path}`);
  }
}
