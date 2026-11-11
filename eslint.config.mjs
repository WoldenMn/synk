import js from "@eslint/js";
import prettier from "eslint-config-prettier/flat";
import { defineConfig } from "eslint/config";
import jsxA11y from "eslint-plugin-jsx-a11y";
import reactHooks from "eslint-plugin-react-hooks";
import globals from "globals";
import tseslint from "typescript-eslint";

const TESTS = ["src/**/*.test.ts", "src/**/*.test.tsx"];

export default defineConfig([
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "src-tauri/**",
      "_archive/**",
      "_backup/**",
      "logs/**",
      "audits/**",
    ],
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      jsxA11y.flatConfigs.recommended,
    ],
    plugins: { "react-hooks": reactHooks },
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.browser, ...globals.node },
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
    linterOptions: { reportUnusedDisableDirectives: "error" },
    rules: {
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "@typescript-eslint/no-empty-function": "error",
      "no-empty": "error",
      "no-console": "error",
      "no-inner-declarations": "error",
      "jsx-a11y/no-autofocus": "error",
      "jsx-a11y/click-events-have-key-events": "error",
      "jsx-a11y/no-static-element-interactions": "error",
      "jsx-a11y/label-has-associated-control": "error",
      "jsx-a11y/heading-has-content": "error",
      "jsx-a11y/control-has-associated-label": [
        "error",
        {
          labelAttributes: ["label"],
          controlComponents: ["Button"],
          ignoreElements: ["audio", "canvas", "embed", "input", "textarea", "tr", "video"],
          ignoreRoles: [
            "grid",
            "listbox",
            "menu",
            "menubar",
            "radiogroup",
            "row",
            "tablist",
            "toolbar",
            "tree",
            "treegrid",
          ],
          depth: 5,
        },
      ],
      "no-restricted-syntax": [
        "error",
        {
          selector: "CallExpression[callee.object.name='toast'][callee.property.name='error']",
          message:
            "Passer par reportError (journal et toast de 10 s), ou notifyError pour une erreur deja consignee.",
        },
        {
          selector:
            "CallExpression[callee.property.name=/^(substring|slice|substr)$/][callee.object.callee.property.name='toISOString']",
          message:
            "Date du jour en UTC (la veille entre 0 h et 2 h a Paris) : utiliser nowDate() de @/lib/dates.",
        },
      ],
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              importNames: ["invoke"],
              message:
                "Passer par le api.ts de la feature (hooks/appReady/api.ts pour le demarrage).",
            },
          ],
        },
      ],
      "max-lines-per-function": ["error", { max: 150, skipBlankLines: true, skipComments: true }],
      complexity: ["error", 15],
      "max-params": ["error", 5],
    },
  },
  {
    files: ["src/lib/**/*.{ts,tsx}"],
    ignores: [...TESTS, "src/lib/reportError.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              importNames: ["invoke"],
              message:
                "Passer par le api.ts de la feature (hooks/appReady/api.ts pour le demarrage).",
            },
          ],
          patterns: [
            {
              group: ["@/features/*", "@/components/*", "@/pages/*", "@/hooks/*", "@/stores/*"],
              message:
                "lib est la couche la plus basse : elle ne depend ni des features, ni des composants, ni des pages (seule exception : reportError.ts, qui ecrit le journal d'erreurs).",
            },
          ],
        },
      ],
    },
  },
  {
    files: [...TESTS, "src/lib/reportError.ts"],
    rules: { "no-restricted-syntax": "off" },
  },
  {
    files: [...TESTS, "src/**/api.ts", "src/test-setup.ts"],
    rules: { "no-restricted-imports": "off" },
  },
  {
    files: [...TESTS, "src/test-setup.ts"],
    rules: {
      "max-lines-per-function": "off",
      complexity: "off",
      "max-params": "off",
      "no-console": "off",
      "@typescript-eslint/no-non-null-assertion": "off",
    },
  },
  {
    files: ["scripts/**/*.{mjs,js}"],
    extends: [js.configs.recommended],
    languageOptions: { ecmaVersion: "latest", sourceType: "module", globals: { ...globals.node } },
    linterOptions: { reportUnusedDisableDirectives: "error" },
  },
  prettier,
]);
