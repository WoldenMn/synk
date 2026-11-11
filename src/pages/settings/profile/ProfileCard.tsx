import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { User } from "lucide-react";
import { getUserProfile, updateUserProfile } from "@/features/profile/api";
import { useLatestBodyMeasure } from "@/features/bodyMeasures/hooks";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { reportError } from "@/lib/reportError";
import { ProfileFields } from "@/components/shared/ProfileFields";

export function ProfileCard() {
  const queryClient = useQueryClient();

  const { data: profile } = useQuery({
    queryKey: ["user_profile"],
    queryFn: getUserProfile,
    staleTime: 5 * 60 * 1000,
  });
  const { data: latestWeight } = useLatestBodyMeasure();
  const [profileAge, setProfileAge] = useState<string>("");
  const [profileSex, setProfileSex] = useState<string>("");
  const [profileHeight, setProfileHeight] = useState<string>("");

  useEffect(() => {
    if (profile) {
      setProfileAge(profile.age != null ? String(profile.age) : "");
      setProfileSex(profile.sex || "");
      setProfileHeight(profile.height_cm != null ? String(profile.height_cm) : "");
    }
  }, [profile]);

  async function handleSaveProfile() {
    const age = profileAge ? Number(profileAge) : null;
    const sex = profileSex || null;
    const heightCm = profileHeight ? Number(profileHeight) : null;
    try {
      await updateUserProfile(age, sex, heightCm);
      queryClient.invalidateQueries({ queryKey: ["user_profile"] });
      toast.success("Profil mis à jour");
    } catch (e) {
      reportError(e, {
        source: "mutation",
        context: "update_user_profile",
        prefix: "Échec de la sauvegarde du profil : ",
      });
    }
  }

  const bmi = (() => {
    if (!latestWeight?.weight || !profileHeight) return null;
    const hm = Number(profileHeight) / 100;
    if (hm <= 0) return null;
    return latestWeight.weight / (hm * hm);
  })();

  const fcMax = profileAge ? 220 - Number(profileAge) : null;

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <User className="h-5 w-5" />
          Profil
        </CardTitle>
        <CardDescription>Vos informations personnelles pour les calculs de santé</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <ProfileFields
          idPrefix="profile"
          age={profileAge}
          sex={profileSex}
          height={profileHeight}
          onAgeChange={setProfileAge}
          onSexChange={setProfileSex}
          onHeightChange={setProfileHeight}
          gapClass="gap-4"
        />
        <div className="flex items-center justify-between">
          <div className="flex gap-4 text-sm text-muted-foreground">
            {bmi != null && (
              <span>
                IMC : <span className="font-medium text-foreground">{bmi.toFixed(1)}</span>
              </span>
            )}
            {fcMax != null && (
              <span>
                FC max : <span className="font-medium text-foreground">{fcMax} bpm</span>
              </span>
            )}
          </div>
          <Button variant="outline" size="sm" onClick={handleSaveProfile}>
            Enregistrer
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
