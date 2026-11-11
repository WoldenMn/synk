import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { FormField } from "../types";
import { Select } from "@/components/ui/select";

interface FormFieldRendererProps {
  field: FormField;
  value: string;
  inputId: string;
  error?: string;
  onChange: (raw: string) => void;
}

type SelectFormField = Extract<FormField, { type: "select" }>;

function inputTypeFor(field: FormField): string {
  switch (field.type) {
    case "datetime":
      return "datetime-local";
    case "date":
      return "date";
    case "time":
      return "time";
    case "number":
    case "decimal":
      return "number";
    default:
      return "text";
  }
}

function numericPropsFor(field: FormField) {
  if (field.type !== "number" && field.type !== "decimal") return {};
  return {
    min: field.min,
    max: field.max,
    step:
      field.step ??
      (field.type === "decimal" ? (field.decimals ? 1 / Math.pow(10, field.decimals) : 0.1) : 1),
  };
}

function FieldError({ id, error }: { id: string; error?: string }) {
  return error ? (
    <p id={`${id}-error`} className="text-sm text-destructive-text">
      {error}
    </p>
  ) : null;
}

function SelectField({
  field,
  value,
  inputId,
  labelText,
  error,
  onChange,
}: {
  field: SelectFormField;
  value: string;
  inputId: string;
  labelText: string;
  error?: string;
  onChange: (raw: string) => void;
}) {
  return (
    <div className="space-y-2">
      <Label htmlFor={inputId}>{labelText}</Label>
      <Select
        id={inputId}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        required={field.required}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${inputId}-error` : undefined}
        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
      >
        {field.options.map((opt) => (
          <option key={String(opt.value)} value={String(opt.value)}>
            {opt.label}
          </option>
        ))}
      </Select>
      <FieldError id={inputId} error={error} />
    </div>
  );
}

export function FormFieldRenderer({
  field,
  value,
  inputId,
  error,
  onChange,
}: FormFieldRendererProps) {
  const unit = field.type === "number" || field.type === "decimal" ? field.unit : undefined;
  const labelText = `${field.label}${unit ? ` (${unit})` : ""}${field.required ? " *" : ""}`;

  if (field.type === "select") {
    return (
      <SelectField
        field={field}
        value={value}
        inputId={inputId}
        labelText={labelText}
        error={error}
        onChange={onChange}
      />
    );
  }

  const textProps = field.type === "text" && field.maxLength ? { maxLength: field.maxLength } : {};

  return (
    <div className="space-y-2">
      <Label htmlFor={inputId}>{labelText}</Label>
      <Input
        id={inputId}
        type={inputTypeFor(field)}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={field.placeholder}
        required={field.required}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${inputId}-error` : undefined}
        {...numericPropsFor(field)}
        {...textProps}
      />
      <FieldError id={inputId} error={error} />
    </div>
  );
}
