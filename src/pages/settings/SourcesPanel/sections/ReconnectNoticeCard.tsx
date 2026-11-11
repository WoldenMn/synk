import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { TriangleAlert } from "lucide-react";
import type { InvalidatedSecrets, SourceInfo } from "@/features/sources/types";
import {
  reconnectReason,
  reconnectSteps,
  reconnectTitle,
  type SourceStates,
} from "@/features/sources/reconnectNotice";

interface ReconnectNoticeCardProps {
  secrets: InvalidatedSecrets | null;
  sources: SourceInfo[] | undefined;
}

function statesOf(sources: SourceInfo[] | undefined): SourceStates {
  return Object.fromEntries(
    (sources ?? []).map((s) => [
      s.id,
      { connected: s.connected, credentialsConfigured: s.credentials_configured },
    ])
  );
}

export function ReconnectNoticeCard({ secrets, sources }: ReconnectNoticeCardProps) {
  const steps = reconnectSteps(secrets, statesOf(sources));

  if (!secrets || steps.length === 0) return null;

  return (
    <Card role="alert" className="border-warning/40 bg-warning/5">
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <TriangleAlert className="h-5 w-5 text-warning-text" aria-hidden="true" />
          {reconnectTitle(secrets)}
        </CardTitle>
        <CardDescription>{reconnectReason(secrets)}</CardDescription>
      </CardHeader>
      <CardContent>
        <ol className="list-decimal space-y-1 pl-5 text-sm">
          {steps.map((step) => (
            <li key={step}>{step}</li>
          ))}
        </ol>
      </CardContent>
    </Card>
  );
}
