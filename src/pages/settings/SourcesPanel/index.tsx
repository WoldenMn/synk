import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { getInvalidatedSecrets } from "@/features/sources/api";
import { useSources } from "@/features/sources/hooks";
import type { InvalidatedSecrets } from "@/features/sources/types";
import { cn } from "@/lib/utils";
import { logError } from "@/lib/logger";
import { PrioritiesTab } from "./PrioritiesTab";
import { ReconnectNoticeCard } from "./sections/ReconnectNoticeCard";
import { SourceCard } from "./sections/SourceCard";
import { DevicesCard } from "./sections/DevicesCard";
import { ImportCard } from "./sections/ImportCard";
import { SyncIntervalCard } from "./sections/SyncIntervalCard";

const TABS = [
  { id: "sources", label: "Sources" },
  { id: "priorities", label: "Priorités" },
] as const;

type TabId = (typeof TABS)[number]["id"];

function TabBar({ active, onSelect }: { active: TabId; onSelect: (id: TabId) => void }) {
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});

  function onKeyDown(event: KeyboardEvent, index: number) {
    const jump = event.key === "Home" ? 0 : event.key === "End" ? TABS.length - 1 : null;
    const step = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
    if (step === 0 && jump === null) return;
    event.preventDefault();
    const next = TABS[jump ?? (index + step + TABS.length) % TABS.length];
    if (!next) return;
    onSelect(next.id);
    refs.current[next.id]?.focus();
  }

  return (
    <div role="tablist" aria-label="Sources" className="flex gap-1 border-b">
      {TABS.map((tab, index) => (
        <button
          key={tab.id}
          ref={(el) => {
            refs.current[tab.id] = el;
          }}
          type="button"
          role="tab"
          id={`sources-tab-${tab.id}`}
          aria-selected={active === tab.id}
          aria-controls="sources-tabpanel"
          tabIndex={active === tab.id ? 0 : -1}
          onClick={() => onSelect(tab.id)}
          onKeyDown={(e) => onKeyDown(e, index)}
          className={cn(
            "-mb-px border-b-2 px-3 py-2 text-sm font-medium",
            active === tab.id
              ? "border-primary text-primary-text"
              : "border-transparent text-muted-foreground hover:text-foreground"
          )}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}

export function SourcesPanel() {
  const { data: sources } = useSources();
  const [tab, setTab] = useState<TabId>("sources");
  const [invalidatedSecrets, setInvalidatedSecrets] = useState<InvalidatedSecrets | null>(null);

  useEffect(() => {
    getInvalidatedSecrets()
      .then(setInvalidatedSecrets)
      .catch((err) => logError("get_invalidated_secrets failed:", err));
  }, []);

  const withingsConnected = sources?.some((s) => s.id === "withings" && s.connected) ?? false;

  return (
    <>
      <ReconnectNoticeCard secrets={invalidatedSecrets} sources={sources} />
      <TabBar active={tab} onSelect={setTab} />
      <div
        role="tabpanel"
        id="sources-tabpanel"
        aria-labelledby={`sources-tab-${tab}`}
        className="space-y-6"
      >
        {tab === "sources" ? (
          <>
            {sources
              ?.filter((source) => source.kind === "oauth")
              .map((source) => (
                <SourceCard key={source.id} source={source} />
              ))}
            <ImportCard sources={sources?.filter((source) => source.kind === "file") ?? []} />
            <DevicesCard visible={withingsConnected} />
            <SyncIntervalCard />
          </>
        ) : (
          <PrioritiesTab />
        )}
      </div>
    </>
  );
}
