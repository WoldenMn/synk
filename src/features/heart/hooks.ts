import { useQuery } from "@tanstack/react-query";
import { getHeartRecords, getLatestHeartRecord } from "./api";

export function useHeartData(from: string, to: string) {
  return useQuery({
    queryKey: ["heart", from, to],
    queryFn: () => getHeartRecords(from, to),
    staleTime: 5 * 60 * 1000,
    enabled: !!from && !!to,
  });
}

export function useLatestHeart() {
  return useQuery({
    queryKey: ["heart", "latest"],
    queryFn: getLatestHeartRecord,
    staleTime: 5 * 60 * 1000,
  });
}
