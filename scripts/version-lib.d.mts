export const VERSION_FILES: string[];
export function isSemver(version: string): boolean;
export function readVersions(read: (path: string) => string): Record<string, string | undefined>;
export function bumpText(path: string, text: string, version: string): string;
export function disagreements(
  versions: Record<string, string | undefined>
): Record<string, string | undefined>;
