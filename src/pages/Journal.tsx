import { useState } from "react";
import { BookOpen, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { JournalForm } from "./journal/JournalForm";
import { JournalEntry } from "./journal/JournalEntry";
import { LoadingSkeleton, EmptyState } from "@/components/shared";
import {
  useJournalEntries,
  useUpsertJournalEntry,
  useDeleteJournalEntry,
} from "@/features/journal/hooks";
import { useConfirm } from "@/components/ui/confirm-dialog";
import { toast } from "sonner";
import { reportError, notifyError } from "@/lib/reportError";
import { nowDate } from "@/lib/dates";

export default function Journal() {
  const { data: entries = [], isLoading } = useJournalEntries();
  const upsert = useUpsertJournalEntry();
  const remove = useDeleteJournalEntry();
  const confirm = useConfirm();

  const [showForm, setShowForm] = useState(false);
  const [date, setDate] = useState(nowDate);
  const [content, setContent] = useState("");
  const [tags, setTags] = useState("");
  const [visible, setVisible] = useState(30);

  async function handleSave() {
    if (!content.trim()) {
      notifyError("Le contenu ne peut pas être vide");
      return;
    }
    try {
      await upsert.mutateAsync({ date, content, tags: tags || null });
      toast.success("Entrée enregistrée");
      setContent("");
      setTags("");
      setShowForm(false);
    } catch (e) {
      reportError(e, { source: "mutation", context: "upsert_journal_entry" });
    }
  }

  async function handleDelete(id: number) {
    if (!(await confirm({ title: "Supprimer cette entrée ?", destructive: true }))) return;
    try {
      await remove.mutateAsync(id);
      toast.success("Supprimée");
    } catch (e) {
      reportError(e, { source: "mutation", context: "delete_journal_entry" });
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div>
          <p className="text-muted-foreground">
            Notes quotidiennes pour mettre vos données en contexte
          </p>
        </div>
        <Button onClick={() => setShowForm(!showForm)}>
          <Plus className="mr-2 h-4 w-4" />
          Nouvelle entrée
        </Button>
      </div>

      {showForm && (
        <JournalForm
          date={date}
          setDate={setDate}
          content={content}
          setContent={setContent}
          tags={tags}
          setTags={setTags}
          saving={upsert.isPending}
          onCancel={() => setShowForm(false)}
          onSave={handleSave}
        />
      )}

      {isLoading ? (
        <LoadingSkeleton />
      ) : entries.length === 0 ? (
        <EmptyState
          icon={<BookOpen className="h-12 w-12" />}
          title="Pas encore d'entrée"
          description="Tenez votre journal pour mieux comprendre vos données."
        />
      ) : (
        <div className="space-y-3">
          {entries.slice(0, visible).map((entry, idx) => (
            <JournalEntry key={entry.id} entry={entry} index={idx} onDelete={handleDelete} />
          ))}
          {entries.length > visible && (
            <div className="flex justify-center">
              <Button variant="outline" onClick={() => setVisible((v) => v + 30)}>
                Afficher 30 entrées de plus
              </Button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
