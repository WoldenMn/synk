import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Settings } from "lucide-react";
import { toast } from "sonner";
import { notifyError, reportError } from "@/lib/reportError";
import { useAuditPurgeConfig, useSetAuditPurgeConfig } from "@/features/audit/hooks";
import type { AuditPurgeConfig } from "@/features/audit/types";

const DEFAULT_CONFIG: AuditPurgeConfig = {
  retention_days: 180,
  max_entries: 10000,
  enabled: true,
};

export function AutoPurgeSection() {
  const { data: purgeConfig } = useAuditPurgeConfig();
  const setConfigMutation = useSetAuditPurgeConfig();
  const [cfg, setCfg] = useState<AuditPurgeConfig>(DEFAULT_CONFIG);

  useEffect(() => {
    if (purgeConfig) setCfg(purgeConfig);
  }, [purgeConfig]);

  function handleSaveConfig() {
    if (cfg.retention_days < 0 || cfg.max_entries < 0) {
      notifyError("Les valeurs doivent être positives");
      return;
    }
    setConfigMutation.mutate(cfg, {
      onSuccess: () => toast.success("Configuration enregistrée"),
      onError: (e) =>
        reportError(e, { source: "mutation", context: "audit_purge_config", prefix: "Échec : " }),
    });
  }

  return (
    <div className="border-t pt-3">
      <p className="text-sm font-medium mb-2 flex items-center gap-1">
        <Settings className="h-4 w-4" />
        Purge automatique au démarrage
      </p>
      <div className="space-y-2">
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={cfg.enabled}
            onChange={(e) => setCfg({ ...cfg, enabled: e.target.checked })}
          />
          Activer (purge au démarrage si les conditions sont remplies)
        </label>
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label htmlFor="auto-purge-retention" className="text-xs text-muted-foreground">
              Retention max (jours)
            </label>
            <Input
              id="auto-purge-retention"
              type="number"
              min={0}
              value={cfg.retention_days}
              onChange={(e) => setCfg({ ...cfg, retention_days: Number(e.target.value) || 0 })}
              disabled={!cfg.enabled}
            />
          </div>
          <div>
            <label htmlFor="auto-purge-max" className="text-xs text-muted-foreground">
              Max entrées
            </label>
            <Input
              id="auto-purge-max"
              type="number"
              min={0}
              value={cfg.max_entries}
              onChange={(e) => setCfg({ ...cfg, max_entries: Number(e.target.value) || 0 })}
              disabled={!cfg.enabled}
            />
          </div>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={handleSaveConfig}
          disabled={setConfigMutation.isPending}
        >
          {setConfigMutation.isPending ? "Enregistrement..." : "Enregistrer"}
        </Button>
      </div>
    </div>
  );
}
