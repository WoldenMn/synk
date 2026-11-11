import { getDashboardCards } from "@/features/dashboard/api";
import { DASHBOARD_CARDS_KEY } from "@/features/dashboard/hooks";
import { rangeBounds } from "@/hooks/useTimeRange";
import type { QueryClient } from "@tanstack/react-query";

export async function prefetchDashboard(qc: QueryClient): Promise<void> {
  const { from, to } = rangeBounds("30d");
  await qc.prefetchQuery({
    queryKey: [DASHBOARD_CARDS_KEY, from, to],
    queryFn: () => getDashboardCards(from, to),
  });
}
