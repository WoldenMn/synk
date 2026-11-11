import { useEffect, useRef, type RefObject } from "react";
import { useLocation } from "react-router-dom";
import { PAGE_TITLES } from "@/lib/constants";

export function useRouteAnnouncement(
  main: RefObject<HTMLElement | null>,
  heading: RefObject<HTMLHeadingElement | null>
): void {
  const { pathname } = useLocation();
  const firstRender = useRef(true);

  useEffect(() => {
    const title = PAGE_TITLES[pathname];
    document.title = title ? `${title} | Synk` : "Synk";
    if (firstRender.current) {
      firstRender.current = false;
      return;
    }
    if (main.current) main.current.scrollTop = 0;
    heading.current?.focus();
  }, [pathname, main, heading]);
}
