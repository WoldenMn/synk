import { Skeleton } from "@/components/ui/skeleton";

interface LoadingSkeletonProps {
  rows?: number;
  height?: string;
}

export function LoadingSkeleton({ rows = 5, height = "h-10" }: LoadingSkeletonProps) {
  return (
    <div className="space-y-2 p-2" role="status" aria-busy="true">
      <span className="sr-only">Chargement en cours</span>
      {[...Array(rows)].map((_, i) => (
        <Skeleton key={i} className={`${height} w-full`} />
      ))}
    </div>
  );
}

interface ChartSkeletonProps {
  height?: number;
}

export function ChartSkeleton({ height = 320 }: ChartSkeletonProps) {
  return (
    <div role="status" aria-busy="true" className="w-full">
      <span className="sr-only">Chargement du graphique</span>
      <Skeleton className="w-full" style={{ height }} />
    </div>
  );
}

interface StatCardGridSkeletonProps {
  count?: number;
}

export function StatCardGridSkeleton({ count = 4 }: StatCardGridSkeletonProps) {
  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4" role="status" aria-busy="true">
      <span className="sr-only">Chargement des statistiques</span>
      {[...Array(count)].map((_, i) => (
        <Skeleton key={i} className="h-28 w-full" />
      ))}
    </div>
  );
}

interface TableSkeletonProps {
  rows?: number;
}

export function TableSkeleton({ rows = 8 }: TableSkeletonProps) {
  return (
    <div className="space-y-2" role="status" aria-busy="true">
      <span className="sr-only">Chargement du tableau</span>
      <Skeleton className="h-9 w-full" />
      {[...Array(rows)].map((_, i) => (
        <Skeleton key={i} className="h-12 w-full" />
      ))}
    </div>
  );
}
