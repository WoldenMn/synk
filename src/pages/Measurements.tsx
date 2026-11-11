import { useState } from "react";
import { ManualMeasurementForm } from "@/features/manualMeasurements/components/ManualMeasurementForm";
import { useMeasurementTypes, useSetMeasurementGoal } from "@/features/manualMeasurements/hooks";
import { MeasurementGrid } from "./measurements/MeasurementGrid";
import { MeasurementDetail } from "./measurements/MeasurementDetail";
import { SizesView } from "./measurements/SizesView";
import { SheetView } from "./measurements/SheetView";
import { GoalDialog } from "./measurements/GoalDialog";
import { ManageTypesDialog } from "./measurements/ManageTypesDialog";
import { NewTypeDialog } from "./measurements/NewTypeDialog";
import { ViewTabs, type MeasurementsView } from "./measurements/ViewTabs";

export default function Measurements() {
  const [selectedTypeId, setSelectedTypeId] = useState<number | null>(null);
  const [view, setView] = useState<MeasurementsView>("sheet");
  const [formOpen, setFormOpen] = useState(false);
  const [goalDialogOpen, setGoalDialogOpen] = useState(false);
  const [newTypeDialogOpen, setNewTypeDialogOpen] = useState(false);
  const [manageOpen, setManageOpen] = useState(false);
  const [goalValue, setGoalValue] = useState<string>("");
  const [goalDirection, setGoalDirection] = useState<string>("decrease");

  const { data: types, isLoading: typesLoading } = useMeasurementTypes();
  const setGoalMutation = useSetMeasurementGoal();

  function handleSaveGoal() {
    if (!selectedTypeId) return;
    const val = goalValue ? Number(goalValue) : null;
    setGoalMutation.mutate(
      {
        typeId: selectedTypeId,
        targetValue: val,
        targetDirection: val != null ? goalDirection : null,
      },
      { onSuccess: () => setGoalDialogOpen(false) }
    );
  }

  function handleDeleteGoal() {
    if (!selectedTypeId) return;
    setGoalMutation.mutate(
      { typeId: selectedTypeId, targetValue: null, targetDirection: null },
      { onSuccess: () => setGoalDialogOpen(false) }
    );
  }

  return (
    <>
      <ManualMeasurementForm open={formOpen} onOpenChange={setFormOpen} />

      <GoalDialog
        goalDialogOpen={goalDialogOpen}
        setGoalDialogOpen={setGoalDialogOpen}
        goalValue={goalValue}
        setGoalValue={setGoalValue}
        goalDirection={goalDirection}
        setGoalDirection={setGoalDirection}
        handleDeleteGoal={handleDeleteGoal}
        handleSaveGoal={handleSaveGoal}
      />

      <ManageTypesDialog
        manageOpen={manageOpen}
        setManageOpen={setManageOpen}
        onAddType={() => setNewTypeDialogOpen(true)}
      />

      <NewTypeDialog
        newTypeDialogOpen={newTypeDialogOpen}
        setNewTypeDialogOpen={setNewTypeDialogOpen}
      />

      {selectedTypeId ? (
        <MeasurementDetail
          typeId={selectedTypeId}
          types={types || []}
          onBack={() => setSelectedTypeId(null)}
          onAddEntry={() => setFormOpen(true)}
          onOpenGoal={() => {
            const t = types?.find((t) => t.id === selectedTypeId);
            if (t) {
              setGoalValue(t.target_value != null ? String(t.target_value) : "");
              setGoalDirection(t.target_direction || "decrease");
            }
            setGoalDialogOpen(true);
          }}
        />
      ) : (
        <div className="space-y-6">
          <ViewTabs view={view} setView={setView} />

          {view === "sheet" ? (
            <SheetView />
          ) : view === "sizes" ? (
            <SizesView />
          ) : (
            <MeasurementGrid
              types={types}
              isLoading={typesLoading}
              onSelect={setSelectedTypeId}
              onAddEntry={() => setFormOpen(true)}
              onManage={() => setManageOpen(true)}
            />
          )}
        </div>
      )}
    </>
  );
}
