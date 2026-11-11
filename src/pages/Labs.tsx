import { useState, useMemo } from "react";
import { PageHeader } from "@/components/shared";
import { useLabResults, useAddLabResult, useDeleteLabResult } from "@/features/labs/hooks";
import type { LabResult } from "@/features/labs/types";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { useTimeRange } from "@/hooks/useTimeRange";
import { nowDate } from "@/lib/dates";
import { isInRange } from "./Labs/helpers";
import { emptyLabDraft, parseLabDraft, type LabDraft, type LabDraftErrors } from "./Labs/labDraft";
import { AddLabForm } from "./Labs/sections/AddLabForm";
import { LabStatsCards } from "./Labs/sections/LabStatsCards";
import { LabAnalysesList } from "./Labs/sections/LabAnalysesList";

export default function Labs() {
  const { timeRange, setTimeRange, from, to } = useTimeRange();
  const { data: results = [], isLoading } = useLabResults(from, to);
  const addResult = useAddLabResult();
  const deleteResult = useDeleteLabResult();
  const [showForm, setShowForm] = useState(false);
  const [expandedTest, setExpandedTest] = useState<string | null>(null);
  const confirm = useConfirm();
  const [draft, setDraft] = useState<LabDraft>(() => emptyLabDraft(nowDate()));
  const [errors, setErrors] = useState<LabDraftErrors>({});

  const groupedByTest = useMemo(() => {
    const map = new Map<string, LabResult[]>();
    for (const r of results) {
      const existing = map.get(r.test_name) ?? [];
      existing.push(r);
      map.set(r.test_name, existing);
    }
    return map;
  }, [results]);

  const totalTests = groupedByTest.size;
  const outOfRangeCount = Array.from(groupedByTest.values()).filter((items) => {
    const latest = items[0];
    if (latest === undefined) return false;
    return isInRange(latest.value, latest.reference_min, latest.reference_max) === false;
  }).length;

  function handleAdd() {
    const parsed = parseLabDraft(draft);
    if (!parsed.ok) {
      setErrors(parsed.errors);
      return;
    }
    setErrors({});
    addResult.mutate(parsed.input, {
      onSuccess: () => {
        setShowForm(false);
        setDraft(emptyLabDraft(nowDate()));
      },
    });
  }

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer ce résultat ?", destructive: true }))) return;
    deleteResult.mutate(id);
  }

  return (
    <div className="space-y-6">
      <PageHeader
        timeRange={timeRange}
        onTimeRangeChange={setTimeRange}
        onAdd={() => setShowForm(true)}
        addLabel="Résultat"
      />

      {showForm && (
        <AddLabForm
          draft={draft}
          errors={errors}
          onChange={setDraft}
          onCancel={() => setShowForm(false)}
          onSubmit={handleAdd}
        />
      )}

      {totalTests > 0 && (
        <LabStatsCards
          totalTests={totalTests}
          totalResults={results.length}
          outOfRangeCount={outOfRangeCount}
        />
      )}

      <LabAnalysesList
        loading={isLoading}
        groupedByTest={groupedByTest}
        expandedTest={expandedTest}
        setExpandedTest={setExpandedTest}
        onDelete={handleDelete}
      />
    </div>
  );
}
