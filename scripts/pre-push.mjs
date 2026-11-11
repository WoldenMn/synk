import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { isTreeVerified, pushedMainCommits } from "./verify-lib.mjs";

const commits = pushedMainCommits(readFileSync(0, "utf-8"));
if (commits.length === 0) process.exit(0);

function readVerifyLog() {
  try {
    return readFileSync("logs/verify.log", "utf-8");
  } catch {
    return "";
  }
}
const log = readVerifyLog();

for (const commit of commits) {
  const tree = execFileSync("git", ["rev-parse", `${commit}^{tree}`], { encoding: "utf-8" }).trim();
  if (!isTreeVerified(log, tree)) {
    console.error(
      `[pre-push] push de main refusé : l'arbre ${tree} du commit ${commit} n'a aucun passage complet réussi dans logs/verify.log.`
    );
    console.error("[pre-push] Lancer « npm run verify » sur un arbre propre, puis repousser.");
    process.exit(1);
  }
  console.log(`[pre-push] arbre ${tree} vérifié au niveau complet`);
}
