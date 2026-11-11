import { useState, useId } from "react";
import { markOnboardingDone } from "@/lib/onboarding";
import { StepLayout } from "./shared/StepLayout";
import { WelcomeStep } from "./steps/WelcomeStep";
import { SourcesStep } from "./steps/SourcesStep";
import { ProfileStep } from "./steps/ProfileStep";
import { GoalsStep } from "./steps/GoalsStep";

type Step = "welcome" | "sources" | "profile" | "goals";
const STEP_ORDER: Step[] = ["welcome", "sources", "profile", "goals"];

export function OnboardingWizard({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState<Step>("welcome");
  const titleId = useId();
  const stepIndex = STEP_ORDER.indexOf(step);
  const isLast = stepIndex === STEP_ORDER.length - 1;

  function finishOnboarding() {
    markOnboardingDone();
    onComplete();
  }

  function next() {
    const following = STEP_ORDER[stepIndex + 1];
    if (isLast || following === undefined) {
      finishOnboarding();
    } else {
      setStep(following);
    }
  }

  function jumpTo(target: Step) {
    setStep(target);
  }

  return (
    <StepLayout
      step={stepIndex + 1}
      totalSteps={STEP_ORDER.length}
      titleId={titleId}
      onDismiss={finishOnboarding}
    >
      {step === "welcome" && (
        <WelcomeStep onComplete={next} onSkipAll={finishOnboarding} titleId={titleId} />
      )}
      {step === "sources" && (
        <SourcesStep onComplete={() => jumpTo("profile")} onSkip={next} titleId={titleId} />
      )}
      {step === "profile" && <ProfileStep onComplete={next} onSkip={next} titleId={titleId} />}
      {step === "goals" && (
        <GoalsStep onComplete={finishOnboarding} onSkip={finishOnboarding} titleId={titleId} />
      )}
    </StepLayout>
  );
}
