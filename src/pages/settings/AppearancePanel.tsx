import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Tooltip } from "@/components/ui/tooltip";
import { Sun, Moon, PanelLeft, Eye, EyeOff } from "lucide-react";
import { NAV_ITEMS } from "@/lib/constants";
import { useUiStore, COLOR_THEMES, type ColorTheme } from "@/stores/uiStore";

const COLOR_THEME_LABELS: Record<ColorTheme, string> = {
  indigo: "Indigo",
  emerald: "Emeraude",
  rose: "Rose",
  amber: "Ambre",
  slate: "Ardoise",
};

const PERMANENT_ROUTES = ["/", "/export", "/settings"];

function NavigationCard() {
  const hiddenRoutes = useUiStore((s) => s.hiddenRoutes);
  const toggleRoute = useUiStore((s) => s.toggleRoute);

  const togglableItems = NAV_ITEMS.filter((item) => !PERMANENT_ROUTES.includes(item.route));

  const allHidden = togglableItems.every((item) => hiddenRoutes.includes(item.route));

  function handleToggleAll() {
    if (allHidden) {
      togglableItems.forEach((item) => {
        if (hiddenRoutes.includes(item.route)) toggleRoute(item.route);
      });
    } else {
      togglableItems.forEach((item) => {
        if (!hiddenRoutes.includes(item.route)) toggleRoute(item.route);
      });
    }
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle as="h3" className="flex items-center gap-2">
          <PanelLeft className="h-5 w-5" />
          Navigation
        </CardTitle>
        <CardDescription>Masquez les onglets que vous n'utilisez pas</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {togglableItems.map((item) => {
          const isHidden = hiddenRoutes.includes(item.route);
          return (
            <button
              key={item.route}
              type="button"
              role="switch"
              aria-checked={!isHidden}
              onClick={() => toggleRoute(item.route)}
              className="flex w-full items-center justify-between rounded-lg border p-3 transition-colors hover:bg-accent/15"
            >
              <span
                className={`text-sm font-medium ${isHidden ? "text-muted-foreground line-through" : ""}`}
              >
                {item.label}
              </span>
              {isHidden ? (
                <EyeOff className="h-4 w-4 text-muted-foreground" aria-hidden="true" />
              ) : (
                <Eye className="h-4 w-4 text-foreground" aria-hidden="true" />
              )}
            </button>
          );
        })}
        <Separator />
        <Button variant="ghost" size="sm" className="w-full" onClick={handleToggleAll}>
          {allHidden ? "Tout afficher" : "Tout masquer"}
        </Button>
      </CardContent>
    </Card>
  );
}

export function AppearancePanel() {
  const theme = useUiStore((s) => s.theme);
  const toggleTheme = useUiStore((s) => s.toggleTheme);
  const colorTheme = useUiStore((s) => s.colorTheme);
  const setColorTheme = useUiStore((s) => s.setColorTheme);

  return (
    <>
      <NavigationCard />

      <Card>
        <CardHeader>
          <CardTitle as="h3" className="flex items-center gap-2">
            {theme === "dark" ? <Moon className="h-5 w-5" /> : <Sun className="h-5 w-5" />}
            Apparence
          </CardTitle>
          <CardDescription>Personnalisez l'apparence de l'application</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Thème</p>
              <p className="text-sm text-muted-foreground">
                {theme === "dark" ? "Mode sombre" : "Mode clair"}
              </p>
            </div>
            <Button variant="outline" onClick={toggleTheme}>
              {theme === "dark" ? (
                <>
                  <Sun className="mr-2 h-4 w-4" />
                  Mode clair
                </>
              ) : (
                <>
                  <Moon className="mr-2 h-4 w-4" />
                  Mode sombre
                </>
              )}
            </Button>
          </div>
          <Separator />
          <div>
            <p className="text-sm font-medium mb-2">Couleur</p>
            <div className="flex gap-3">
              {COLOR_THEMES.map((ct) => {
                const label = COLOR_THEME_LABELS[ct as ColorTheme];
                const isActive = colorTheme === ct;
                return (
                  <Tooltip key={ct} content={label} side="top">
                    <button
                      onClick={() => setColorTheme(ct as ColorTheme)}
                      className={`h-8 w-8 rounded-full bg-primary ${theme === "dark" ? "dark" : ""} theme-${ct} transition-all ${
                        isActive
                          ? "ring-2 ring-offset-2 ring-offset-background ring-foreground scale-110"
                          : "hover:scale-105 opacity-70 hover:opacity-100"
                      }`}
                      aria-label={`Thème ${label}`}
                      aria-pressed={isActive}
                    />
                  </Tooltip>
                );
              })}
            </div>
          </div>
        </CardContent>
      </Card>
    </>
  );
}
