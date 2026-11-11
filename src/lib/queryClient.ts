import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query";
import { reportError } from "./reportError";

export function createToastThrottle(everyMs: number) {
  const last = new Map<string, number>();
  return (key: string, now: number): boolean => {
    const previous = last.get(key);
    if (previous !== undefined && now - previous < everyMs) return false;
    last.set(key, now);
    return true;
  };
}

export function createQueryClient(): QueryClient {
  const allowToast = createToastThrottle(60_000);
  const client: QueryClient = new QueryClient({
    mutationCache: new MutationCache({
      onSuccess: () => {
        void client.invalidateQueries({ queryKey: ["dashboardCards"] });
      },
    }),
    queryCache: new QueryCache({
      onError: (error, query) => {
        const shell = query.meta?.surface === "shell";
        if (!shell && query.state.data === undefined) return;
        const key = JSON.stringify(query.queryKey);
        reportError(error, { source: "query", context: key, toast: allowToast(key, Date.now()) });
      },
    }),
    defaultOptions: {
      queries: {
        staleTime: 1000 * 60 * 5,
        gcTime: 1000 * 60 * 30,
        refetchOnWindowFocus: false,
        retry: 1,
        throwOnError: (_error, query) => query.state.data === undefined,
      },
    },
  });
  return client;
}
