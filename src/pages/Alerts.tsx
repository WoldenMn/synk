import { useState } from "react";
import { toast } from "sonner";
import {
  useAlerts,
  useAlertEvents,
  useAddAlert,
  useDeleteAlert,
  useCheckAlerts,
  useMarkAlertsSeen,
} from "@/features/alerts/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { AlertsList } from "./Alerts/sections/AlertsList";
import { AlertEvents } from "./Alerts/sections/AlertEvents";
import {
  emptyAlertDraft,
  parseAlertDraft,
  type AlertDraft,
  type AlertDraftErrors,
} from "./Alerts/alertDraft";

export default function Alerts() {
  const { data: alerts = [] } = useAlerts();
  const { data: events = [] } = useAlertEvents();
  const addAlertMut = useAddAlert();
  const deleteAlertMut = useDeleteAlert();
  const checkAlertsMut = useCheckAlerts();
  const markSeenMut = useMarkAlertsSeen();
  const [showForm, setShowForm] = useState(false);
  const confirm = useConfirm();
  const [draft, setDraft] = useState<AlertDraft>(emptyAlertDraft);
  const [errors, setErrors] = useState<AlertDraftErrors>({});

  function handleAdd() {
    const parsed = parseAlertDraft(draft);
    if (!parsed.ok) {
      setErrors(parsed.errors);
      return;
    }
    setErrors({});
    addAlertMut.mutate(parsed.input, {
      onSuccess: () => {
        setShowForm(false);
        setDraft(emptyAlertDraft());
      },
    });
  }

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cette alerte ?", destructive: true }))) return;
    deleteAlertMut.mutate(id);
  }

  function handleCheck() {
    checkAlertsMut.mutate(undefined, {
      onSuccess: (triggered) => {
        if (triggered.length > 0) {
          toast.warning(`${triggered.length} alerte(s) déclenchée(s)`);
        } else {
          toast.success("Aucune alerte déclenchée");
        }
      },
    });
  }

  function handleMarkSeen() {
    markSeenMut.mutate(undefined, {
      onSuccess: () => toast.success("Événements marqués comme lus"),
    });
  }

  return (
    <div className="grid items-start gap-6 xl:grid-cols-2">
      <AlertsList
        alerts={alerts}
        showForm={showForm}
        draft={draft}
        errors={errors}
        onToggleForm={() => setShowForm(!showForm)}
        onDraftChange={setDraft}
        onAdd={handleAdd}
        onDelete={handleDelete}
        onCheck={handleCheck}
      />
      <AlertEvents alerts={alerts} events={events} onMarkSeen={handleMarkSeen} />
    </div>
  );
}
