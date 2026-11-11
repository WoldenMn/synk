import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { ArrowRight, SkipForward, User } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { updateUserProfile } from "@/features/profile/api";
import { logWarn } from "@/lib/logger";
import { ProfileFields } from "@/components/shared/ProfileFields";

interface ProfileStepProps {
  onComplete: () => void;
  onSkip: () => void;
  titleId: string;
}

export function ProfileStep({ onComplete, onSkip, titleId }: ProfileStepProps) {
  const [profileAge, setProfileAge] = useState("");
  const [profileSex, setProfileSex] = useState("");
  const [profileHeight, setProfileHeight] = useState("");
  const queryClient = useQueryClient();

  async function handleSaveProfile() {
    const age = profileAge ? Number(profileAge) : null;
    const sex = profileSex || null;
    const heightCm = profileHeight ? Number(profileHeight) : null;
    try {
      await updateUserProfile(age, sex, heightCm);
      queryClient.invalidateQueries({ queryKey: ["user_profile"] });
    } catch (e) {
      logWarn("[ProfileStep] updateUserProfile échoué (non bloquant)", e);
      toast.warning("Données non enregistrées, vous pourrez les compléter dans Paramètres.");
    }
    onComplete();
  }

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10">
          <User className="h-8 w-8 text-primary-text" />
        </div>
        <h2 id={titleId} className="text-xl font-bold">
          Votre profil
        </h2>
        <p className="text-sm text-muted-foreground">
          Ces informations permettent de calculer votre IMC et FC max.
        </p>
      </div>
      <ProfileFields
        idPrefix="onboarding"
        age={profileAge}
        sex={profileSex}
        height={profileHeight}
        onAgeChange={setProfileAge}
        onSexChange={setProfileSex}
        onHeightChange={setProfileHeight}
      />
      <div className="flex gap-3">
        <Button variant="ghost" className="flex-1" onClick={onSkip}>
          <SkipForward className="mr-2 h-4 w-4" />
          Passer
        </Button>
        <Button className="flex-1" onClick={handleSaveProfile}>
          Continuer
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
