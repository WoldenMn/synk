import axe from "axe-core";

axe.configure({
  rules: axe.getRules(["cat.color"]).map(({ ruleId }) => ({ id: ruleId, enabled: false })),
});

export async function expectNoAxeViolations(
  container: Element,
  options: axe.RunOptions = {}
): Promise<void> {
  const results = await axe.run(container, options);
  const found = results.violations.map(
    (v) => `${v.id} (${v.impact ?? "impact inconnu"}) : ${v.nodes.length} noeud(s), ${v.help}`
  );
  if (found.length > 0) {
    throw new Error(`Violations d'accessibilité :\n${found.join("\n")}`);
  }
}
