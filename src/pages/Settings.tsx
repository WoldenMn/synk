import { SourcesPanel } from "./settings/SourcesPanel";
import { ProfilePanel } from "./settings/ProfilePanel";
import { AppearancePanel } from "./settings/AppearancePanel";
import { DataPathCard } from "./settings/DataPathCard";
import { BackupsCard } from "./settings/BackupsCard";
import { DataMetricsCard } from "./settings/DataMetricsCard";

function ColumnTitle({ id, children }: { id: string; children: string }) {
  return (
    <h2 id={id} className="text-xs font-semibold uppercase tracking-widest text-muted-foreground">
      {children}
    </h2>
  );
}

export default function Settings() {
  return (
    <div className="grid grid-cols-1 items-start gap-6 xl:grid-cols-2">
      <section aria-labelledby="settings-sources" className="space-y-6">
        <ColumnTitle id="settings-sources">Sources</ColumnTitle>
        <SourcesPanel />
      </section>
      <section aria-labelledby="settings-app" className="space-y-6">
        <ColumnTitle id="settings-app">Application</ColumnTitle>
        <ProfilePanel />
        <AppearancePanel />
        <DataPathCard />
        <BackupsCard />
        <DataMetricsCard />
      </section>
    </div>
  );
}
