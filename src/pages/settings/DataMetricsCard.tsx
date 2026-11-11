import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Database } from "lucide-react";
import { StatsBlock } from "./dataMetrics/StatsBlock";
import { PurgeSection } from "./dataMetrics/PurgeSection";
import { AutoPurgeSection } from "./dataMetrics/AutoPurgeSection";

export function DataMetricsCard() {
  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <Database className="h-5 w-5" />
          Données et observabilité
        </CardTitle>
        <CardDescription>Statistiques locales et purge du journal d'audit</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <StatsBlock />
        <PurgeSection />
        <AutoPurgeSection />
      </CardContent>
    </Card>
  );
}
