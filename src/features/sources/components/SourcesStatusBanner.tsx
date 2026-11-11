import { useNavigate } from "react-router-dom";
import { TriangleAlert, Unplug } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useAnySourceConnected, useSources } from "../hooks";
import { reauthNotices } from "../reauth";

const BOX =
  "flex items-center gap-3 rounded-lg border border-warning/40 bg-warning/10 px-4 py-2 text-sm";

export function SourcesStatusBanner() {
  const navigate = useNavigate();
  const { data: connected, isError } = useAnySourceConnected();
  const { data: sources } = useSources();
  const notices = reauthNotices(sources);

  const showStatus = connected === false || isError;
  if (!showStatus && notices.length === 0) return null;

  const Icon = isError ? TriangleAlert : Unplug;
  const message = isError
    ? "État des sources inconnu : la vérification a échoué."
    : "Aucune source n'est connectée : vos données s'arrêtent à la dernière synchronisation.";

  return (
    <div className="flex flex-col gap-2">
      {showStatus && (
        <div role="status" className={BOX}>
          <Icon className="h-4 w-4 shrink-0 text-warning-text" aria-hidden="true" />
          <span>{message}</span>
          <Button
            size="sm"
            variant="outline"
            className="ml-2 h-8"
            onClick={() => navigate("/settings")}
          >
            Connecter une source
          </Button>
        </div>
      )}
      {notices.map((notice) => (
        <div key={notice.id} role="status" className={BOX}>
          <TriangleAlert className="h-4 w-4 shrink-0 text-warning-text" aria-hidden="true" />
          <span>{notice.text}</span>
          <Button
            size="sm"
            variant="outline"
            className="ml-2 h-8"
            onClick={() => navigate("/settings")}
          >
            Ouvrir les sources
          </Button>
        </div>
      ))}
    </div>
  );
}
