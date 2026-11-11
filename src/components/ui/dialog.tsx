import * as React from "react";
import { createPortal } from "react-dom";
import { FocusScope } from "@radix-ui/react-focus-scope";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface DialogContextValue {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  titleId: string;
  descriptionId: string;
}

const DialogContext = React.createContext<DialogContextValue>({
  open: false,
  // eslint-disable-next-line @typescript-eslint/no-empty-function -- valeur par defaut du context, remplacee par le Provider
  onOpenChange: () => {},
  titleId: "",
  descriptionId: "",
});

interface DialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: React.ReactNode;
}

function Dialog({ open, onOpenChange, children }: DialogProps) {
  const titleId = React.useId();
  const descriptionId = React.useId();
  return (
    <DialogContext.Provider value={{ open, onOpenChange, titleId, descriptionId }}>
      {children}
    </DialogContext.Provider>
  );
}

function DialogContent({ className, children, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  const { open, onOpenChange, titleId, descriptionId } = React.useContext(DialogContext);

  React.useEffect(() => {
    if (open) {
      document.body.style.overflow = "hidden";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [open]);

  React.useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape") onOpenChange(false);
    }
    if (open) {
      document.addEventListener("keydown", handleEscape);
    }
    return () => document.removeEventListener("keydown", handleEscape);
  }, [open, onOpenChange]);

  if (!open) return null;

  return createPortal(
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="fixed inset-0 bg-black/50 backdrop-blur-xs"
        role="button"
        tabIndex={-1}
        aria-label="Fermer le dialogue"
        onClick={() => onOpenChange(false)}
        onKeyDown={(e) => {
          if (e.key === "Escape" || e.key === "Enter" || e.key === " ") {
            onOpenChange(false);
          }
        }}
      />
      <FocusScope trapped loop asChild>
        <div
          role="dialog"
          aria-modal="true"
          aria-labelledby={titleId}
          aria-describedby={descriptionId}
          className={cn(
            "relative z-50 w-full max-w-lg rounded-lg border bg-background p-6 shadow-lg",
            "animate-in fade-in-0 zoom-in-95",
            className
          )}
          {...props}
        >
          <button
            onClick={() => onOpenChange(false)}
            aria-label="Fermer"
            className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          >
            <X className="h-4 w-4" />
          </button>
          {children}
        </div>
      </FocusScope>
    </div>,
    document.body
  );
}

function DialogHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)}
      {...props}
    />
  );
}

function DialogTitle({
  className,
  id: idProp,
  ...props
}: React.HTMLAttributes<HTMLHeadingElement>) {
  const { titleId } = React.useContext(DialogContext);
  return (
    // eslint-disable-next-line jsx-a11y/heading-has-content -- primitive shadcn : le contenu vient de {...props} / children
    <h2
      id={idProp ?? titleId}
      className={cn("text-lg font-semibold leading-none tracking-tight", className)}
      {...props}
    />
  );
}

function DialogDescription({
  className,
  id: idProp,
  ...props
}: React.HTMLAttributes<HTMLParagraphElement>) {
  const { descriptionId } = React.useContext(DialogContext);
  return (
    <p
      id={idProp ?? descriptionId}
      className={cn("text-sm text-muted-foreground", className)}
      {...props}
    />
  );
}

function DialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2", className)}
      {...props}
    />
  );
}

export { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter };
