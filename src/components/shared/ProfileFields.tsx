import { cn } from "@/lib/utils";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";

const FIELD_CLASS =
  "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2";

interface ProfileFieldsProps {
  idPrefix: string;
  age: string;
  sex: string;
  height: string;
  onAgeChange: (value: string) => void;
  onSexChange: (value: string) => void;
  onHeightChange: (value: string) => void;
  gapClass?: string;
}

export function ProfileFields({
  idPrefix,
  age,
  sex,
  height,
  onAgeChange,
  onSexChange,
  onHeightChange,
  gapClass = "gap-3",
}: ProfileFieldsProps) {
  return (
    <div className={cn("grid grid-cols-3", gapClass)}>
      <div className="space-y-1.5">
        <label htmlFor={`${idPrefix}-age`} className="text-sm font-medium">
          Âge
        </label>
        <Input
          id={`${idPrefix}-age`}
          type="number"
          min={1}
          max={150}
          placeholder="ex. 30"
          value={age}
          onChange={(e) => onAgeChange(e.target.value)}
          className={FIELD_CLASS}
        />
      </div>
      <div className="space-y-1.5">
        <label htmlFor={`${idPrefix}-sex`} className="text-sm font-medium">
          Sexe
        </label>
        <Select
          id={`${idPrefix}-sex`}
          value={sex}
          onChange={(e) => onSexChange(e.target.value)}
          className={FIELD_CLASS}
        >
          <option value="">--</option>
          <option value="homme">Homme</option>
          <option value="femme">Femme</option>
        </Select>
      </div>
      <div className="space-y-1.5">
        <label htmlFor={`${idPrefix}-height`} className="text-sm font-medium">
          Taille (cm)
        </label>
        <Input
          id={`${idPrefix}-height`}
          type="number"
          min={50}
          max={300}
          step={0.1}
          placeholder="ex. 175"
          value={height}
          onChange={(e) => onHeightChange(e.target.value)}
          className={FIELD_CLASS}
        />
      </div>
    </div>
  );
}
