import { createElement, isValidElement } from "react";

interface EmptyStateProps {
  icon: React.ElementType | React.ReactNode;
  title: string;
  description: string;
  action?: React.ReactNode;
  compact?: boolean;
}

/**
 * Detecte si `value` est un component React valide (function, class, forwardRef, memo, lazy, string).
 * `typeof === "function"` seul ne suffit PAS : les Lucide icons sont des `forwardRef(...)` qui ont
 * `typeof === "object"` avec un champ `$$typeof`. Tenter de rendre `{forwardRef}` directement
 * declenche React error #31 ("object with keys $$typeof, render, displayName").
 */
function isComponentType(value: unknown): value is React.ElementType {
  if (typeof value === "function") return true; // FunctionComponent / class
  if (typeof value === "string") return true; // intrinsic HTML tags ("div", etc.)
  if (typeof value === "object" && value !== null && "$$typeof" in value) return true;
  return false;
}

export function EmptyState({ icon, title, description, action, compact }: EmptyStateProps) {
  const renderedIcon = isValidElement(icon)
    ? icon
    : isComponentType(icon)
      ? createElement(icon, { className: "h-8 w-8 text-muted-foreground" })
      : (icon as React.ReactNode);

  return (
    <div
      className={`flex flex-col items-center justify-center text-center ${compact ? "py-4" : "py-12"}`}
    >
      <div className="rounded-full bg-muted p-4 mb-4 text-muted-foreground">{renderedIcon}</div>
      <h2 className="text-lg font-semibold mb-1">{title}</h2>
      <p className="text-sm text-muted-foreground max-w-sm">{description}</p>
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
