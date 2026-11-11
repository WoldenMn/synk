import { Save } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

interface Props {
  date: string;
  setDate: (v: string) => void;
  content: string;
  setContent: (v: string) => void;
  tags: string;
  setTags: (v: string) => void;
  saving: boolean;
  onCancel: () => void;
  onSave: () => void;
}

export function JournalForm({
  date,
  setDate,
  content,
  setContent,
  tags,
  setTags,
  saving,
  onCancel,
  onSave,
}: Props) {
  return (
    <div className="animate-fade-in-up">
      <Card>
        <CardHeader>
          <CardTitle>Nouvelle entrée</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid gap-2">
            <label htmlFor="journal-date" className="text-sm font-medium">
              Date
            </label>
            <Input
              id="journal-date"
              type="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
            />
          </div>
          <div className="grid gap-2">
            <label htmlFor="journal-content" className="text-sm font-medium">
              Contenu
            </label>
            <Textarea
              id="journal-content"
              className="min-h-[120px] w-full rounded-md border border-input bg-background p-3 text-sm focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              placeholder="Comment vous sentez-vous aujourd'hui ? Des événements marquants ?"
              value={content}
              onChange={(e) => setContent(e.target.value)}
            />
          </div>
          <div className="grid gap-2">
            <label htmlFor="journal-tags" className="text-sm font-medium">
              Tags (optionnel)
            </label>
            <Input
              id="journal-tags"
              placeholder="sport, sommeil, stress..."
              value={tags}
              onChange={(e) => setTags(e.target.value)}
            />
          </div>
          <div className="flex gap-2 justify-end">
            <Button variant="outline" onClick={() => onCancel()}>
              Annuler
            </Button>
            <Button onClick={onSave} disabled={saving}>
              <Save className="mr-2 h-4 w-4" />
              Enregistrer
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
