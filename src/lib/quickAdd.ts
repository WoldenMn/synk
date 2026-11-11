const QUICK_ADD_ROUTES = ["/", "/body", "/blood-pressure", "/measurements"];

export function showsQuickAdd(pathname: string): boolean {
  return QUICK_ADD_ROUTES.includes(pathname);
}
