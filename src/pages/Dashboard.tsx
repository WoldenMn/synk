import { useBodyMeasures } from "@/features/bodyMeasures/hooks";
import { useBloodPressureData } from "@/features/bloodPressure/hooks";
import { useSleepData } from "@/features/sleep/hooks";
import { useActivityData } from "@/features/activity/hooks";
import { useTimeRange } from "@/hooks/useTimeRange";
import { TimeRangeSelector } from "@/components/charts/TimeRangeSelector";
import { formatISO, startOfDay, parseISO } from "date-fns";
import { KPICards } from "./Dashboard/sections/KPICards";
import { TrendCharts } from "./Dashboard/sections/TrendCharts";
import { SourcesStatusBanner } from "@/features/sources/components/SourcesStatusBanner";

export default function Dashboard() {
  const { timeRange, setTimeRange, from, to, compareFrom, compareTo, setCompareRange } =
    useTimeRange();

  const compareActive = compareFrom != null && compareTo != null;
  const cFrom = compareActive ? formatISO(startOfDay(parseISO(compareFrom))) : "";
  const cTo = compareActive ? formatISO(startOfDay(parseISO(compareTo))) : "";

  const { data: bodyMeasures, isLoading: bodyLoading } = useBodyMeasures(from, to);
  const { data: bpData, isLoading: bpLoading } = useBloodPressureData(from, to);
  const { data: sleepData, isLoading: sleepLoading } = useSleepData(from, to);
  const { data: activityData, isLoading: activityLoading } = useActivityData(from, to);
  const isLoading = bodyLoading || bpLoading || sleepLoading || activityLoading;

  const { data: cmpBody } = useBodyMeasures(cFrom, cTo, compareActive);
  const { data: cmpBP } = useBloodPressureData(cFrom, cTo, compareActive);
  const { data: cmpSleep } = useSleepData(cFrom, cTo, compareActive);
  const { data: cmpActivity } = useActivityData(cFrom, cTo, compareActive);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <SourcesStatusBanner />
        <div className="ml-auto">
          <TimeRangeSelector
            value={timeRange}
            onChange={setTimeRange}
            compareFrom={compareFrom}
            compareTo={compareTo}
            onCompareChange={setCompareRange}
          />
        </div>
      </div>

      <KPICards
        from={from}
        to={to}
        bodyMeasures={bodyMeasures}
        bpData={bpData}
        sleepData={sleepData}
        activityData={activityData}
        cmpBody={cmpBody}
        cmpBP={cmpBP}
        cmpSleep={cmpSleep}
        cmpActivity={cmpActivity}
        compareActive={compareActive}
      />

      <TrendCharts
        bodyMeasures={bodyMeasures}
        bpData={bpData}
        sleepData={sleepData}
        activityData={activityData}
        isLoading={isLoading}
        timeRange={timeRange}
        onShowAll={() => setTimeRange("all")}
      />
    </div>
  );
}
