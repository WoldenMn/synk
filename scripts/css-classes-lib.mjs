export function cssClassNames(css) {
  const names = new Set();
  const withoutBlocks = css.replace(/\{[^{}]*\}/g, "{}");
  for (const match of withoutBlocks.matchAll(/\.((?:\\.|[A-Za-z_-])(?:\\.|[A-Za-z0-9_-])*)/g)) {
    names.add(match[1].replace(/\\(.)/g, "$1"));
  }
  return [...names].sort();
}
