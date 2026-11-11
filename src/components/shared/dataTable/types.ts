export interface Column<T> {
  key: string;
  label: string;
  render?: (row: T) => React.ReactNode;
  searchText?: (row: T) => string;
}

export type SortDir = "asc" | "desc";
