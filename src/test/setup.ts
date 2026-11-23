// Vitest setup file
// Mock Tauri APIs for testing
import { vi } from "vitest";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/window", () => {
  // One shared window object so tests can inspect calls made by the app.
  const currentWindow = {
    onCloseRequested: vi.fn().mockResolvedValue(() => {}),
    setTitle: vi.fn().mockResolvedValue(undefined),
    destroy: vi.fn().mockResolvedValue(undefined),
  };
  return { getCurrentWindow: () => currentWindow };
});

// Mock @tauri-apps/plugin-dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: vi.fn(() => "macos"),
}));

// Mock @tauri-apps/plugin-process
vi.mock("@tauri-apps/plugin-process", () => ({
  exit: vi.fn(),
  relaunch: vi.fn(),
}));

// Mock @tauri-apps/api/app
vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("1.0.0"),
  getName: vi.fn().mockResolvedValue("Catalyst"),
  getTauriVersion: vi.fn().mockResolvedValue("2.0.0"),
}));

// Mock @tauri-apps/plugin-opener
vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
  openPath: vi.fn(),
  revealItemInDir: vi.fn(),
}));

// jsdom implements <dialog> but not its modal API. Components open native
// dialogs with showModal(), so provide the minimal open/close behaviour the
// browser has. Individual suites may still override these per test.
if (typeof HTMLDialogElement !== "undefined") {
  if (typeof HTMLDialogElement.prototype.showModal !== "function") {
    HTMLDialogElement.prototype.showModal = function (this: HTMLDialogElement) {
      this.open = true;
    };
  }
  if (typeof HTMLDialogElement.prototype.show !== "function") {
    HTMLDialogElement.prototype.show = function (this: HTMLDialogElement) {
      this.open = true;
    };
  }
  if (typeof HTMLDialogElement.prototype.close !== "function") {
    HTMLDialogElement.prototype.close = function (this: HTMLDialogElement, returnValue?: string) {
      if (!this.open) return;
      this.open = false;
      if (returnValue !== undefined) this.returnValue = returnValue;
      this.dispatchEvent(new Event("close"));
    };
  }
}
