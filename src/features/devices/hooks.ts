import { useQuery } from "@tanstack/react-query";
import { getWithingsDevices } from "./api";

export function useDevices() {
  return useQuery({
    queryKey: ["devices"],
    queryFn: getWithingsDevices,
    staleTime: 5 * 60 * 1000,
  });
}
