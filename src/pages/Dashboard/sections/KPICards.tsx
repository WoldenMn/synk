import { useEntranceOnce } from "@/hooks/useEntranceOnce";
import { useKpiState, type KpiInput } from "../useKpiState";
import { KPICard } from "./KPICard";

export function KPICards(props: KpiInput) {
  const animateEntrance = useEntranceOnce("dashboard");
  const { goals, deltas, trends, cards } = useKpiState(props);

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {cards.map((card, index) => (
        <KPICard
          key={card.id}
          card={card}
          index={index}
          animateEntrance={animateEntrance}
          trend={trends[card.id]}
          goals={goals}
          delta={deltas[card.id]}
          compareActive={props.compareActive}
        />
      ))}
    </div>
  );
}
