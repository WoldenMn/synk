import { cn } from "@/lib/utils";

interface SynkLogoProps {
  size?: number;
  className?: string;
}

// Meme marque que l'icone de l'application (src-tauri/icons/source/synk-icon-32.svg),
// dessinee sur la grille de 32. La tuile suit la couleur d'accent choisie dans
// Parametres, la ou l'icone Windows reste indigo.
export function SynkLogo({ size = 28, className }: SynkLogoProps) {
  return (
    <svg
      viewBox="0 0 32 32"
      width={size}
      height={size}
      aria-hidden="true"
      focusable="false"
      className={cn("shrink-0", className)}
    >
      <rect x="1" y="1" width="30" height="30" rx="7" fill="hsl(var(--primary))" />
      <g
        fill="none"
        stroke="hsl(var(--primary-foreground))"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M 8.5 13 A 8.5 8.5 0 0 1 23.5 13" strokeWidth="2.5" />
        <path d="M 23.5 19 A 8.5 8.5 0 0 1 8.5 19" strokeWidth="2.5" />
        <path d="M 5 16 H 12 L 14 12.5 L 16.5 20.5 L 19 11.5 L 20.5 16 H 27" strokeWidth="2.25" />
      </g>
    </svg>
  );
}
