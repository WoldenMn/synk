import * as React from "react";
import { createPortal } from "react-dom";
import { cn } from "@/lib/utils";

interface TooltipProps extends React.HTMLAttributes<HTMLDivElement> {
  content: string;
  side?: "top" | "bottom" | "left" | "right";
}

interface Anchor {
  left: number;
  top: number;
}

const GAP = 8;

const SIDE_CLASSES = {
  top: "-translate-x-1/2 -translate-y-full",
  bottom: "-translate-x-1/2",
  left: "-translate-x-full -translate-y-1/2",
  right: "-translate-y-1/2",
};

function anchorFor(rect: DOMRect, side: NonNullable<TooltipProps["side"]>): Anchor {
  switch (side) {
    case "top":
      return { left: rect.left + rect.width / 2, top: rect.top - GAP };
    case "bottom":
      return { left: rect.left + rect.width / 2, top: rect.bottom + GAP };
    case "left":
      return { left: rect.left - GAP, top: rect.top + rect.height / 2 };
    default:
      return { left: rect.right + GAP, top: rect.top + rect.height / 2 };
  }
}

function Tooltip({ children, content, side = "right", className, ...props }: TooltipProps) {
  const [anchor, setAnchor] = React.useState<Anchor | null>(null);
  const ref = React.useRef<HTMLDivElement>(null);

  const show = () => {
    if (ref.current) setAnchor(anchorFor(ref.current.getBoundingClientRect(), side));
  };
  const hide = () => setAnchor(null);

  return (
    <div
      ref={ref}
      className={cn("relative inline-flex", className)}
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}
      {...props}
    >
      {children}
      {anchor &&
        content &&
        createPortal(
          <div role="region" aria-label="Infobulle">
            <div
              role="tooltip"
              style={{ left: anchor.left, top: anchor.top }}
              className={cn(
                "fixed z-50 whitespace-nowrap rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md border animate-in fade-in-0 zoom-in-95",
                SIDE_CLASSES[side]
              )}
            >
              {content}
            </div>
          </div>,
          document.body
        )}
    </div>
  );
}

export { Tooltip };
