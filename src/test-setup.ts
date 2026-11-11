import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Default invoke retourne `undefined` resolved (au lieu de undefined synchrone) pour
// eviter les "Cannot read property of undefined" dans les hooks qui attendent une Promise.
// Chaque test devrait override avec mockResolvedValue() pour ses cas, mais ce default
// evite les crash silencieux quand un test oublie de setup le mock.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
  // eslint-disable-next-line @typescript-eslint/no-empty-function -- unsubscribe noop retourne par listen() dans les mocks
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
  revealItemInDir: vi.fn(),
}));
