import { format } from "date-fns";

export function nowDatetimeLocal(): string {
  return format(new Date(), "yyyy-MM-dd'T'HH:mm");
}

export function nowDate(): string {
  return format(new Date(), "yyyy-MM-dd");
}

export function nowTime(): string {
  return format(new Date(), "HH:mm");
}
