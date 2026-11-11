import { useCallback, useRef, useState } from "react";
import type { ReactNode } from "react";
import { ConfirmContext } from "./useConfirm";
import type { ConfirmFn, ConfirmOptions } from "./useConfirm";
import { ConfirmDialog } from "./ConfirmDialog";

interface ConfirmState {
  open: boolean;
  options: ConfirmOptions | null;
}

export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<ConfirmState>({ open: false, options: null });
  const resolverRef = useRef<((value: boolean) => void) | null>(null);

  const settle = useCallback((value: boolean) => {
    const resolver = resolverRef.current;
    resolverRef.current = null;
    setState({ open: false, options: null });
    resolver?.(value);
  }, []);

  const confirm = useCallback<ConfirmFn>((options) => {
    resolverRef.current?.(false);
    return new Promise<boolean>((resolve) => {
      resolverRef.current = resolve;
      setState({ open: true, options });
    });
  }, []);

  return (
    <ConfirmContext.Provider value={confirm}>
      {children}
      <ConfirmDialog
        open={state.open}
        options={state.options}
        onConfirm={() => settle(true)}
        onCancel={() => settle(false)}
      />
    </ConfirmContext.Provider>
  );
}
