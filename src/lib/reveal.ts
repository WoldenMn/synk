import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { logWarn } from "./logger";

export async function revealDataFolder(dir: string): Promise<void> {
  try {
    await revealItemInDir(`${dir}\\synk.db`);
  } catch (e) {
    logWarn("[reveal] synk.db introuvable, ouverture du dossier seul", e);
    await revealItemInDir(dir);
  }
}
