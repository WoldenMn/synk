import { memo } from "react";
import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Scale,
  Heart,
  Moon,
  Footprints,
  Dumbbell,
  HeartPulse,
  Ruler,
  Download,
  Settings,
  ChevronLeft,
  Bell,
  ClipboardList,
  ChartLine,
  Pill,
  FlaskConical,
  Sparkles,
  BookOpen,
  Activity,
  Compass,
  User,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/stores/uiStore";
import { Separator } from "@/components/ui/separator";
import { Tooltip } from "@/components/ui/tooltip";
import { SHORTCUTS } from "@/lib/shortcuts";
import { Button } from "@/components/ui/button";
import { NAV_SECTIONS, NAV_SECONDARY } from "@/lib/constants";
import { SynkLogo } from "@/components/brand/SynkLogo";

const ICONS: Record<string, React.ElementType> = {
  LayoutDashboard,
  Scale,
  Heart,
  Moon,
  Footprints,
  Dumbbell,
  HeartPulse,
  Ruler,
  Download,
  Settings,
  Bell,
  ClipboardList,
  ChartLine,
  Pill,
  FlaskConical,
  Sparkles,
  BookOpen,
  Activity,
  Compass,
  User,
};

interface NavItemData {
  route: string;
  label: string;
  icon: string;
}

function SidebarLink({ item, collapsed }: { item: NavItemData; collapsed: boolean }) {
  const Icon = ICONS[item.icon] ?? LayoutDashboard;
  return (
    <Tooltip content={collapsed ? item.label : ""} side="right" className="flex w-full">
      <NavLink
        to={item.route}
        end={item.route === "/"}
        aria-label={item.label}
        className={({ isActive }) =>
          cn(
            "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
            "hover:bg-accent/15 hover:text-foreground",
            "focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset",
            isActive
              ? "bg-primary/15 text-foreground font-semibold shadow-[inset_3px_0_0_hsl(var(--primary))]"
              : "text-muted-foreground",
            collapsed && "justify-center px-0"
          )
        }
      >
        <Icon className="h-4 w-4 shrink-0" aria-hidden="true" />
        {!collapsed && <span className="truncate">{item.label}</span>}
      </NavLink>
    </Tooltip>
  );
}

function SidebarInner() {
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const hiddenRoutes = useUiStore((s) => s.hiddenRoutes);

  return (
    <aside
      aria-label="Navigation principale"
      className={cn(
        "flex h-screen flex-col border-r bg-card/70 backdrop-blur-2xl transition-[width] duration-300",
        "dark:bg-card/40 dark:border-white/5",
        collapsed ? "w-16" : "w-60"
      )}
    >
      <div
        className={cn(
          "flex h-14 items-center gap-2.5",
          collapsed ? "justify-center px-2" : "justify-between pl-4 pr-3"
        )}
      >
        {collapsed ? (
          <Tooltip content={`Déplier la barre latérale (${SHORTCUTS.sidebar.label})`} side="right">
            <button
              type="button"
              onClick={toggleSidebar}
              aria-label="Déplier la barre latérale"
              aria-keyshortcuts={SHORTCUTS.sidebar.keyshortcuts}
              className="rounded-lg focus-visible:outline-hidden focus-visible:ring-2 focus-visible:ring-ring"
            >
              <SynkLogo size={30} />
            </button>
          </Tooltip>
        ) : (
          <>
            <div className="flex min-w-0 items-center gap-2.5">
              <SynkLogo size={28} />
              <span className="text-lg font-semibold tracking-tight text-foreground">Synk</span>
            </div>
            <Tooltip
              content={`Réduire la barre latérale (${SHORTCUTS.sidebar.label})`}
              side="right"
            >
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 shrink-0 text-muted-foreground"
                onClick={toggleSidebar}
                aria-label="Réduire la barre latérale"
                aria-keyshortcuts={SHORTCUTS.sidebar.keyshortcuts}
              >
                <ChevronLeft className="h-4 w-4" />
              </Button>
            </Tooltip>
          </>
        )}
      </div>

      <nav aria-label="Menu" className="flex flex-1 flex-col gap-0.5 px-2 py-2 overflow-y-auto">
        {NAV_SECTIONS.map((section, sIdx) => {
          const visibleItems = section.items.filter((item) => !hiddenRoutes.includes(item.route));
          if (visibleItems.length === 0) return null;

          return (
            <div key={sIdx}>
              {section.label && !collapsed && (
                <p className="px-3 pt-4 pb-1 text-xs font-semibold uppercase tracking-widest text-muted-foreground">
                  {section.label}
                </p>
              )}
              {section.label && collapsed && <Separator className="my-1.5" />}
              {visibleItems.map((item) => (
                <SidebarLink key={item.route} item={item} collapsed={collapsed} />
              ))}
            </div>
          );
        })}

        <div className="mt-auto pt-2">
          <Separator className="mb-2" />
          {NAV_SECONDARY.map((item) => (
            <SidebarLink key={item.route} item={item} collapsed={collapsed} />
          ))}
        </div>
      </nav>
    </aside>
  );
}

export const Sidebar = memo(SidebarInner);
