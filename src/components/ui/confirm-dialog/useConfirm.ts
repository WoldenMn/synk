import { createContext, useContext } from "react";

export interface ConfirmOptions {
  title: string;
  description?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
}

export type ConfirmFn = (options: ConfirmOptions) => Promise<boolean>;

export const ConfirmContext = createContext<ConfirmFn | null>(null);

export function useConfirm(): ConfirmFn {
  const confirm = useContext(ConfirmContext);
  if (confirm === null) {
    throw new Error("useConfirm doit être utilisé à l'intérieur d'un ConfirmProvider");
  }
  return confirm;
}
