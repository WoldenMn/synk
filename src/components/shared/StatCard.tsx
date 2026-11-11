import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface StatCardProps {
  label: string;
  value: string;
  sub?: React.ReactNode;
  icon?: React.ReactNode;
  glow?: "indigo" | "emerald" | "violet" | "rose" | "amber";
  delay?: number;
}

const GLOW_CLASSES: Record<string, string> = {
  indigo: "hover:glow-indigo",
  emerald: "hover:glow-emerald",
  violet: "hover:glow-violet",
  rose: "hover:glow-rose",
  amber: "hover:glow-amber",
};

export function StatCard({ label, value, sub, icon, glow, delay = 0 }: StatCardProps) {
  return (
    <Card
      className={cn(
        "animate-fade-in-up transition-all duration-300 hover:scale-[1.02]",
        glow && GLOW_CLASSES[glow]
      )}
      style={{ animationDelay: `${delay}ms` }}
    >
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">{label}</CardTitle>
          {icon}
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-2xl font-bold animate-count-up">{value}</p>
        {sub && <div className="mt-1">{sub}</div>}
      </CardContent>
    </Card>
  );
}
