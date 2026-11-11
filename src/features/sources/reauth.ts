import { formatDate } from "@/lib/formatters";
import type { ReauthInfo, SourceInfo } from "./types";

export interface ReauthNotice {
  id: string;
  text: string;
}

export function reauthText(label: string, reauth: ReauthInfo): string {
  const date = formatDate(new Date(reauth.expires_at * 1000).toISOString());
  if (reauth.status === "expired") {
    return `L'autorisation de ${label} a expiré le ${date} : reconnectez-la pour reprendre la synchronisation.`;
  }
  return `L'autorisation de ${label} expire le ${date} : reconnectez-la avant cette date.`;
}

export function reauthNotices(sources: SourceInfo[] | undefined): ReauthNotice[] {
  return (sources ?? []).flatMap((s) =>
    s.connected && s.reauth ? [{ id: s.id, text: reauthText(s.label, s.reauth) }] : []
  );
}
