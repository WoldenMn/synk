import { createElement } from "react";

export const legendLabel = (text: string) =>
  createElement("span", { className: "text-foreground" }, text);

export const axisTick = { fill: "hsl(var(--muted-foreground))", fontSize: 12 } as const;

export const axisTickSmall = { fill: "hsl(var(--muted-foreground))", fontSize: 12 } as const;

export const keepSeriesOrder = () => 0;

export const tooltipProps = {
  itemSorter: keepSeriesOrder,
  contentStyle: {
    backgroundColor: "hsl(var(--card))",
    border: "1px solid hsl(var(--border))",
    borderRadius: "8px",
  },
  itemStyle: { color: "hsl(var(--foreground))" },
  labelStyle: { color: "hsl(var(--foreground))" },
} as const;
