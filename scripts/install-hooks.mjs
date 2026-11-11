import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";

function run(cmd, args) {
  try {
    return { ok: true, out: execFileSync(cmd, args, { encoding: "utf-8" }).trim() };
  } catch (e) {
    return { ok: false, out: e.message };
  }
}

const set = run("git", ["config", "core.hooksPath", ".githooks"]);
const local = run("git", ["config", "--get", "core.hooksPath"]);
const globalDir = run("git", ["config", "--global", "--get", "core.hooksPath"]);
const globalHook = globalDir.ok ? join(globalDir.out, "pre-commit") : "";

console.log(`core.hooksPath local : ${local.ok ? local.out : "ABSENT"}`);
console.log(`hook global : ${globalHook && existsSync(globalHook) ? globalHook : "ABSENT"}`);

if (!set.ok || local.out !== ".githooks" || !globalHook || !existsSync(globalHook)) {
  console.error("Hooks incomplets : un commit sera refuse tant que le hook global manque.");
  process.exit(1);
}
console.log("Hooks en place.");
