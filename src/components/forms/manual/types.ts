export type FieldType = "number" | "decimal" | "text" | "datetime" | "time" | "date" | "select";

export interface FormFieldBase {
  name: string;
  label: string;
  required?: boolean;
  placeholder?: string;
  transform?: (raw: string) => unknown;
}

export interface NumberField extends FormFieldBase {
  type: "number" | "decimal";
  unit?: string;
  decimals?: number;
  min?: number;
  max?: number;
  step?: number;
}

export interface TextField extends FormFieldBase {
  type: "text";
  maxLength?: number;
}

export interface DatetimeField extends FormFieldBase {
  type: "datetime" | "time" | "date";
}

export interface SelectField extends FormFieldBase {
  type: "select";
  options: { value: string | number; label: string }[];
}

export type FormField = NumberField | TextField | DatetimeField | SelectField;

export interface FormConfig<TInput extends Record<string, unknown>> {
  fields: FormField[];
  submit: (data: TInput) => Promise<unknown>;
  defaultValues?: Partial<TInput>;
  successMessage?: string;
  onSuccess?: () => void;
  title?: string;
  description?: string;
  submitLabel?: string;
  cancelLabel?: string;
  invalidateKeys?: string[];
}
