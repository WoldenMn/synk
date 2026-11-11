import { useQuery } from "@tanstack/react-query";
import { getDashboardCards, getDbMetrics } from "./api";

export const DASHBOARD_CARDS_KEY = "dashboardCards";

export function useDbMetrics() {
  return useQuery({
    queryKey: ["db_metrics"],
    queryFn: getDbMetrics,
    staleTime: 60_000,
  });
}

export function useDashboardCards(from: string, to: string) {
  return useQuery({
    queryKey: [DASHBOARD_CARDS_KEY, from, to],
    queryFn: () => getDashboardCards(from, to),
  });
}
