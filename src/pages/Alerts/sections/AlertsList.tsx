import { Bell, Plus, TriangleAlert } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import type { HealthAlert } from "@/features/alerts/types";
import { AlertForm } from "./AlertForm";
import { AlertRows } from "./AlertRows";
import type { AlertDraft, AlertDraftErrors } from "../alertDraft";

interface AlertsListProps {
  alerts: HealthAlert[];
  showForm: boolean;
  draft: AlertDraft;
  errors: AlertDraftErrors;
  onToggleForm: () => void;
  onDraftChange: (data: AlertDraft) => void;
  onAdd: () => void;
  onDelete: (id: number) => void;
  onCheck: () => void;
}

export function AlertsList({
  alerts,
  showForm,
  draft,
  errors,
  onToggleForm,
  onDraftChange,
  onAdd,
  onDelete,
  onCheck,
}: AlertsListProps) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Alertes santé
            </CardTitle>
            <CardDescription>
              Définissez des seuils et recevez des alertes quand une valeur les dépasse
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button variant="outline" size="sm" onClick={onCheck}>
              <TriangleAlert className="mr-2 h-4 w-4" />
              Vérifier
            </Button>
            <Button size="sm" onClick={onToggleForm}>
              <Plus className="mr-2 h-4 w-4" />
              Ajouter
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {showForm && (
          <AlertForm
            draft={draft}
            errors={errors}
            onDraftChange={onDraftChange}
            onToggleForm={onToggleForm}
            onAdd={onAdd}
          />
        )}

        <AlertRows alerts={alerts} onDelete={onDelete} />
      </CardContent>
    </Card>
  );
}
