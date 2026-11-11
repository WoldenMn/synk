import type { BootErrorPayload } from "@/hooks/useAppReady";

export interface BootErrorCopy {
  title: string;
  description: string;
  critical: boolean;
  canRestore: boolean;
  canOpenFolder: boolean;
  canQuit: boolean;
}

export function bootErrorCopy(error: BootErrorPayload): BootErrorCopy {
  const base = { critical: false, canRestore: false, canOpenFolder: false, canQuit: false };
  switch (error.kind) {
    case "DbBusyAfterRetry":
      return {
        ...base,
        title: "La base est verrouillée",
        description:
          "Une autre instance de Synk est probablement encore ouverte. Fermez-la, puis réessayez.",
        canOpenFolder: true,
      };
    case "DbLocked":
      return {
        ...base,
        title: "Base de données inaccessible",
        description:
          "La base de données ne peut pas être ouverte. Réessayez, ou ouvrez le dossier des données pour vérifier le fichier synk.db.",
        canOpenFolder: true,
      };
    case "MigrationFailed":
      return {
        ...base,
        title: `Mise à jour de la base échouée (v${error.version})`,
        description:
          "La mise à jour du schéma a échoué. Une sauvegarde automatique a été faite juste avant : vous pouvez la restaurer.",
        critical: true,
        canRestore: true,
      };
    case "DevOnRealData":
      return {
        ...base,
        title: "Mode développement refusé sur les vraies données",
        description: `Ce build de développement allait ouvrir la base de l'application installée (${error.identifier}). Lancez npm run tauri:dev, qui travaille sur la copie com.synk.health.dev.`,
        critical: true,
        canQuit: true,
      };
    case "BackupFailed":
      return {
        ...base,
        title: "Sauvegarde avant mise à jour impossible",
        description:
          "Synk doit mettre à jour votre base, mais n'a pas pu la sauvegarder d'abord. Rien n'a été modifié. Vérifiez l'espace disque, ou restaurez la dernière sauvegarde si la base est abîmée.",
        critical: true,
        canRestore: true,
        canOpenFolder: true,
      };
    case "AppDataDir":
      return {
        ...base,
        title: "Dossier de données inaccessible",
        description:
          "Synk ne peut pas accéder à son dossier dans AppData. Vérifiez les droits de votre session Windows sur ce dossier.",
        critical: true,
        canQuit: true,
      };
    case "CryptoUnavailable":
      return {
        ...base,
        title: "Stockage sécurisé inaccessible",
        description:
          "L'accès au stockage sécurisé de Windows (Gestionnaire d'identification, Credential Manager) a échoué. Fermez puis rouvrez votre session Windows.",
        critical: true,
      };
    case "CryptoMigrationFailed":
      return {
        ...base,
        title: "Chiffrement des accès aux sources échoué",
        description:
          "Le chiffrement des accès à vos sources a échoué. Une sauvegarde automatique de votre base est conservée dans backups/auto.",
        critical: true,
      };
  }
}
