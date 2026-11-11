import { useEffect, useId, type ReactNode } from "react";
import { FocusScope } from "@radix-ui/react-focus-scope";
import { Card, CardContent } from "@/components/ui/card";

interface StepLayoutProps {
  step: number;
  totalSteps: number;
  children: ReactNode;
  titleId?: string;
  onDismiss?: () => void;
}

export function StepLayout({
  step,
  totalSteps,
  children,
  titleId: titleIdProp,
  onDismiss,
}: StepLayoutProps) {
  const fallbackId = useId();
  const titleId = titleIdProp ?? fallbackId;
  const stepIndex = step - 1;

  useEffect(() => {
    if (!onDismiss) return;
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        e.preventDefault();
        onDismiss?.();
      }
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onDismiss]);

  return (
    <FocusScope trapped loop asChild>
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className="fixed inset-0 z-50 flex items-center justify-center bg-background/95 backdrop-blur-xs"
      >
        <Card className="w-full max-w-md mx-4 shadow-2xl border-primary/20">
          <CardContent className="p-8">
            <p className="sr-only">
              Étape {step} sur {totalSteps}
            </p>
            <div className="flex justify-center gap-2 mb-8" aria-hidden="true">
              {Array.from({ length: totalSteps }).map((_, i) => (
                <div
                  key={i}
                  className={`h-1.5 w-10 rounded-full transition-all ${
                    i <= stepIndex ? "bg-primary" : "bg-muted"
                  }`}
                />
              ))}
            </div>
            {children}
          </CardContent>
        </Card>
      </div>
    </FocusScope>
  );
}
