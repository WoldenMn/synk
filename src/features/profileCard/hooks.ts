import { useQuery, useMutation } from "@tanstack/react-query";
import { toast } from "sonner";
import { getProfileCard, exportProfileCardPdf } from "./api";
import { reportError } from "@/lib/reportError";

export function useProfileCard() {
  return useQuery({ queryKey: ["profileCard"], queryFn: getProfileCard });
}

export function useExportProfileCardPdf() {
  return useMutation({
    mutationFn: exportProfileCardPdf,
    onSuccess: (path) => toast.success(`Carte exportée : ${path}`),
    onError: (err: unknown) => {
      reportError(err, { source: "mutation", context: "export_profile_card_pdf" });
    },
  });
}
