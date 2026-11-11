import { useEffect, useState } from "react";
import { getDbMetrics } from "@/features/dashboard/api";
import { listSources } from "@/features/sources/api";
import { errorDetail } from "@/lib/errors";
import { logWarn } from "@/lib/logger";
import { markOnboardingDone } from "@/lib/onboarding";
import { OnboardingWizard } from ".";
import { needsOnboarding } from "./needsOnboarding";

export function OnboardingGate({ onDone }: { onDone: () => void }) {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    Promise.all([getDbMetrics(), listSources()])
      .then(([metrics, sources]) => {
        if (cancelled) return;
        if (needsOnboarding(metrics, sources)) {
          setOpen(true);
        } else {
          markOnboardingDone();
          onDone();
        }
      })
      .catch((e: unknown) => {
        logWarn("[onboarding] état de l'installation illisible, assistant ouvert", errorDetail(e));
        if (!cancelled) setOpen(true);
      });
    return () => {
      cancelled = true;
    };
  }, [onDone]);

  return open ? <OnboardingWizard onComplete={onDone} /> : null;
}
