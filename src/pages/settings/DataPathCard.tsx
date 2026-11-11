import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Tooltip } from "@/components/ui/tooltip";
import { Database } from "lucide-react";
import { getAppDataDir } from "@/features/settings/api";

export function DataPathCard() {
  const [dbPath, setDbPath] = useState("...");

  useEffect(() => {
    getAppDataDir()
      .then((dir) => setDbPath(dir + "\\synk.db"))
      .catch(() => setDbPath("synk.db"));
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <Database className="h-5 w-5" />
          Données
        </CardTitle>
        <CardDescription>Base de données locale</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <p className="text-sm">Stockage</p>
          <p className="text-sm text-muted-foreground">SQLite (local)</p>
        </div>
        <Separator />
        <div className="flex items-center justify-between gap-4">
          <p className="text-sm shrink-0">Emplacement</p>
          <Tooltip content={dbPath} side="left" className="min-w-0">
            <p className="text-sm text-muted-foreground font-mono truncate text-right">{dbPath}</p>
          </Tooltip>
        </div>
      </CardContent>
    </Card>
  );
}
