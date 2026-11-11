import { SourceBadge } from "./components/SourceBadge";
import { sourceLabel } from "./labels";

export function sourceColumn<T extends { source: string }>() {
  return {
    key: "source",
    label: "Source",
    render: (row: T) => <SourceBadge source={row.source} />,
    searchText: (row: T) => sourceLabel(row.source),
  };
}
