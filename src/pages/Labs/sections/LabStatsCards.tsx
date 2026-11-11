import { Card, CardContent } from "@/components/ui/card";

interface Props {
  totalTests: number;
  totalResults: number;
  outOfRangeCount: number;
}

export function LabStatsCards({ totalTests, totalResults, outOfRangeCount }: Props) {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
      <Card>
        <CardContent className="pt-5 pb-4">
          <p className="text-xs text-muted-foreground">Analyses suivies</p>
          <p className="text-2xl font-bold mt-0.5">{totalTests}</p>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-5 pb-4">
          <p className="text-xs text-muted-foreground">Total résultats</p>
          <p className="text-2xl font-bold mt-0.5">{totalResults}</p>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="pt-5 pb-4">
          <p className="text-xs text-muted-foreground">Hors référence</p>
          <p
            className={`text-2xl font-bold mt-0.5 ${outOfRangeCount > 0 ? "text-destructive-text" : "text-success-text"}`}
          >
            {outOfRangeCount}
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
