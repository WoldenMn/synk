import { ArrowRight } from "lucide-react";
import { SynkLogo } from "@/components/brand/SynkLogo";
import { Button } from "@/components/ui/button";

interface WelcomeStepProps {
  onComplete: () => void;
  onSkipAll: () => void;
  titleId: string;
}

export function WelcomeStep({ onComplete, onSkipAll, titleId }: WelcomeStepProps) {
  return (
    <div className="text-center space-y-6">
      <SynkLogo size={64} className="mx-auto" />
      <div className="space-y-2">
        <h2 id={titleId} className="text-2xl font-bold">
          Bienvenue sur Synk
        </h2>
        <p className="text-muted-foreground">
          Suivez votre santé en connectant vos appareils et vos comptes santé. Poids, sommeil,
          tension et activité, tout au même endroit.
        </p>
      </div>
      <Button className="w-full" size="lg" onClick={onComplete}>
        Commencer
        <ArrowRight className="ml-2 h-4 w-4" />
      </Button>
      <Button variant="ghost" className="w-full" onClick={onSkipAll}>
        Passer l'assistant
      </Button>
    </div>
  );
}
