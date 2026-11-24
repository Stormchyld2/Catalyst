import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ui } from "./lib/stores/ui.svelte";
import { proseSaves } from "./lib/utils/proseSaves";
import { exit } from "@tauri-apps/plugin-process";
import { synopsisSaves } from "./lib/stores/synopsisSaves.svelte";
import { session } from "./lib/stores/session.svelte";
import { currentProject } from "./lib/stores/project.svelte";
import { workspace } from "./lib/stores/workspace.svelte";
import { mockProject, mockScenes, mockChapters } from "./dev/mock-data";
import App from "./App.svelte";
import { shortcuts } from "./lib/stores/shortcuts.svelte";
import { defaultBindings } from "./lib/utils/keyboardShortcuts";
import { EditorialSaves } from "./lib/utils/editorialSaves";
import { trackNativeDialog } from "./lib/utils/modalFocus";

vi.hoisted(() => {
  const values = new Map([
    ["kindling:onboardingCompleted", "true"],
    ["kindling:guidanceEnabled", "false"],
  ]);
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
  });
});
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock("./lib/utils/import", async (original) => ({
  ...(await original<object>()),
  runImport: vi.fn(),
}));

const doc = {
  id: "page",
  scene_id: mockScenes[0].id,
  chapter_id: "chapter",
  chapter_title: "Chapter",
  scene_title: "Original scene",
  beat_title: null,
  prose: "<p>Alice Alice</p>",
  locked: false,
};

beforeEach(async () => {
  shortcuts.bindings = { ...defaultBindings };
  shortcuts.recording = false;
  ui.clearToast();
  vi.mocked(invoke).mockReset();
  vi.mocked(invoke).mockImplementation(async (cmd) =>
    cmd === "get_search_documents" ? [doc] : []
  );
  vi.mocked(listen).mockClear();
  vi.mocked(exit).mockClear();
  vi.mocked(getCurrentWindow().onCloseRequested).mockClear();
  currentProject.setProject(mockProject);
  // Opening a project enters its default (Write) workspace in the Writer OS shell.
  workspace.enterProject();
  currentProject.setCurrentScene({ ...mockScenes[0], planning_status: "undefined" });
  // Drain the real debounce timer before individual tests install fake timers or flush spies.
  await session.flush();
  HTMLElement.prototype.scrollIntoView = vi.fn();
});
afterEach(async () => {
  cleanup();
  vi.mocked(invoke).mockResolvedValue([]);
  await synopsisSaves.flush();
  await proseSaves.flush(mockProject.id);
  await proseSaves.discard(proseSaves.draftsForRecovery(mockProject.id));
  ui.setExpandedBeat(null);
  vi.restoreAllMocks();
  currentProject.setProject(null);
  workspace.leaveProject();
  await session.flush();
  vi.useRealTimers();
});
async function menu(payload: string) {
  const callback = vi.mocked(listen).mock.calls.find(([event]) => event === "menu-event")![1];
  callback({ event: "menu-event", id: 1, payload });
  await tick();
}

it("waits for pending position saves before quitting from the menu", async () => {
  let finish!: () => void;
  vi.spyOn(session, "flush").mockReturnValue(
    new Promise<void>((resolve) => {
      finish = resolve;
    })
  );
  render(App);
  await menu("quit");
  expect(exit).not.toHaveBeenCalled();
  finish();
  await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
});

it("waits for pending position saves in the native close handler", async () => {
  render(App);
  let finish!: () => void;
  vi.spyOn(session, "flush").mockReturnValue(
    new Promise<void>((resolve) => {
      finish = resolve;
    })
  );
  const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
  let closed = false;
  const closing = Promise.resolve(handler({} as never)).then(() => {
    closed = true;
  });
  await tick();
  expect(closed).toBe(false);
  finish();
  await closing;
  expect(closed).toBe(true);
});

it.each(["menu", "native"])(
  "still closes through %s when a position flush rejects",
  async (path) => {
    render(App);
    await tick();
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(session, "flush").mockRejectedValue(new Error("disk full"));
    if (path === "menu") {
      await menu("quit");
      await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
    } else {
      const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
      await expect(handler({} as never)).resolves.toBeUndefined();
    }
    expect(error).toHaveBeenCalledWith(
      "Failed to save writing position before closing:",
      expect.any(Error)
    );
  }
);
it.each([
  ["menu", "Cannot edit a locked scene"],
  ["native", "Cannot edit a locked scene"],
  ["menu", "disk full"],
  ["native", "disk full"],
])("allows explicit quit-and-discard via %s after %s", async (path, failure) => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsavable draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw failure;
    return [];
  });
  const preventDefault = vi.fn();
  if (path === "menu") await menu("quit");
  else {
    const handler = vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0];
    await handler({ preventDefault } as never);
    expect(preventDefault).toHaveBeenCalledOnce();
  }
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).not.toHaveBeenCalled();
  expect(synopsisSaves.failedCount).toBe(1);
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).toHaveBeenCalledWith(0);
  expect(synopsisSaves.getState(mockProject.id, mockScenes[0].id).draft).toBeUndefined();
  await vi.advanceTimersByTimeAsync(2000);
  expect(
    vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "save_scene_synopsis")
  ).toHaveLength(1);
});

it("requires fresh discard confirmation if synopsis drafts change while the quit dialog is open", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  const draft = {
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Original draft",
  };
  synopsisSaves.stage(draft);
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw "disk full";
    return [];
  });
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  synopsisSaves.stage({ ...draft, synopsis: "New changes after the dialog opened" });
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).not.toHaveBeenCalled();
  expect(synopsisSaves.getState(draft.projectId, draft.sceneId).draft?.synopsis).toBe(
    "New changes after the dialog opened"
  );
  expect(screen.getByRole("main").inert).toBe(false);
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
  await vi.advanceTimersByTimeAsync(0);
  expect(exit).toHaveBeenCalledWith(0);
});

it.each([false, true])(
  "flushes writing position before quit-and-discard (position save fails: %s)",
  async (fails) => {
    vi.useFakeTimers();
    currentProject.setChapters(mockChapters);
    render(App);
    await vi.advanceTimersByTimeAsync(0);
    synopsisSaves.stage({
      projectId: mockProject.id,
      sceneId: mockScenes[0].id,
      synopsis: "Locked draft",
    });
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === "save_scene_synopsis") throw "Cannot edit a locked scene";
      return [];
    });
    await menu("quit");
    await vi.advanceTimersByTimeAsync(0);
    let finish!: () => void;
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    const flush = vi.spyOn(session, "flush").mockImplementation(async () => {
      await new Promise<void>((resolve) => {
        finish = resolve;
      });
      if (fails) throw new Error("disk full");
    });
    await fireEvent.click(screen.getByRole("button", { name: "Quit and discard" }));
    await vi.advanceTimersByTimeAsync(0);
    try {
      expect(flush).toHaveBeenCalledOnce();
      expect(exit).not.toHaveBeenCalled();
    } finally {
      finish?.();
    }
    await vi.advanceTimersByTimeAsync(0);
    expect(exit).toHaveBeenCalledWith(0);
    if (fails)
      expect(error).toHaveBeenCalledWith(
        "Failed to save writing position before closing:",
        expect.any(Error)
      );
  }
);

it("keeps errors visible and dismissable within the active quit modal", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  ui.showError("Error before quitting");
  await tick();
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsaved draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw new Error("disk full");
    return [];
  });
  await menu("quit");
  await vi.advanceTimersByTimeAsync(0);
  const confirmation = screen.getByRole("dialog", {
    name: "Quit without saving synopsis changes?",
  });
  expect(confirmation.tagName).toBe("DIALOG");
  expect(confirmation.querySelector('[role="dialog"]')).toBeNull();
  expect(
    screen.getByText("Error before quitting").closest("[data-quit-confirmation]")
  ).toBeTruthy();
  ui.showError("Error during confirmation");
  await tick();
  const dismiss = screen.getByRole("button", { name: "Dismiss error" });
  expect(dismiss.closest("[data-quit-confirmation]")).toBeTruthy();
  expect(screen.getAllByRole("button", { name: "Dismiss error" })).toHaveLength(1);
  await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
  await fireEvent.keyDown(document.activeElement!, { key: "Tab" });
  expect(document.activeElement).toBe(dismiss);
  await fireEvent.click(dismiss);
  expect(screen.queryByText("Error during confirmation")).toBeNull();
  expect(screen.getByRole("button", { name: "Keep editing" })).toBeTruthy();
  ui.showError("Still relevant after cancelling");
  await tick();
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  expect(
    screen.getByText("Still relevant after cancelling").closest("[data-quit-confirmation]")
  ).toBeNull();
  expect(screen.getAllByRole("button", { name: "Dismiss error" })).toHaveLength(1);
  shortcuts.bindings = { ...defaultBindings };
  ui.clearToast();
});

it("blocks retry saving while quit preparation is pending", async () => {
  vi.useFakeTimers();
  currentProject.setChapters(mockChapters);
  synopsisSaves.stage({
    projectId: mockProject.id,
    sceneId: mockScenes[0].id,
    synopsis: "Unsaved draft",
  });
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "save_scene_synopsis") throw new Error("disk full");
    return [];
  });
  await expect(synopsisSaves.flush()).rejects.toThrow();
  render(App);
  await vi.advanceTimersByTimeAsync(0);
  const retry = screen.getByRole("button", {
    name: "Retry all synopsis saves",
  }) as HTMLButtonElement;
  let fail!: (error: Error) => void;
  const flush = vi.spyOn(synopsisSaves, "flush").mockImplementation(
    () =>
      new Promise((_, reject) => {
        fail = reject;
      })
  );
  try {
    await menu("quit");
    await vi.advanceTimersByTimeAsync(0);
    expect(retry.disabled).toBe(true);
    await fireEvent.click(retry);
    expect(flush).toHaveBeenCalledOnce();
    fail(new Error("disk full"));
    await vi.advanceTimersByTimeAsync(0);
  } finally {
    flush.mockRestore();
  }
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  const enabledRetry = screen.getByRole("button", {
    name: "Retry all synopsis saves",
  }) as HTMLButtonElement;
  expect(enabledRetry.disabled).toBe(false);
  vi.mocked(invoke).mockResolvedValue([]);
  await fireEvent.click(enabledRetry);
  await vi.advanceTimersByTimeAsync(0);
  expect(synopsisSaves.failedCount).toBe(0);
});
it("attempts writing saves and offers explicit recovery when an editorial save blocks quitting", async () => {
  const round = {
    id: "quit-review",
    project_id: "project",
    title: "Letter",
    name: "Review",
    brief: "",
    created_at: "today",
    sources: [
      {
        id: "source",
        scene_id: "scene",
        chapter_id: "chapter",
        chapter: "Chapter",
        scene: "Letter",
        mode: "page",
        html: "<p>Eleanor waited.</p>",
        locked: false,
      },
    ],
  };
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === "take_editorial_open_files") return ["/review.catalyst-review"];
    if (cmd === "open_editorial_package")
      return { format: "kindling-editorial", version: 1, kind: "review", round, session: null };
    return [];
  });
  Range.prototype.getClientRects = vi.fn().mockReturnValue([]);
  Range.prototype.getBoundingClientRect = vi
    .fn()
    .mockReturnValue({ left: 0, right: 0, top: 0, bottom: 0 });
  render(App);
  await screen.findByRole("region", { name: "Editorial workspace" });
  await fireEvent.input(screen.getByLabelText("Your name"), { target: { value: "Rowan" } });
  vi.spyOn(EditorialSaves.prototype, "flush").mockRejectedValue(new Error("disk full"));
  const writing = vi.spyOn(proseSaves, "flush").mockResolvedValue([]);
  const synopsis = vi.spyOn(synopsisSaves, "flush").mockResolvedValue(undefined);
  await menu("quit");
  await screen.findByRole("dialog", { name: "Quit without saving review changes?" });
  expect(writing).toHaveBeenCalled();
  expect(synopsis).toHaveBeenCalled();
  expect(exit).not.toHaveBeenCalled();
  await fireEvent.click(screen.getByRole("button", { name: "Keep editing" }));
  expect(localStorage.getItem("kindling.editorial.recovery.quit-review")).toContain("Rowan");
  localStorage.removeItem("kindling.editorial.recovery.quit-review");
});

it.each(["an app dialog", "a file picker", "an import"])(
  "ignores native menu commands while %s is open",
  async (modal) => {
    render(App);
    let finishPicker!: () => void;
    if (modal === "an app dialog") {
      await menu("about");
      expect(screen.getByTestId("about-dialog")).toBeTruthy();
    } else if (modal === "a file picker") {
      void trackNativeDialog(() => new Promise<void>((resolve) => (finishPicker = resolve)));
    } else {
      ui.startImport();
      await tick();
    }

    // Cmd+W and friends arrive as menu events; none may act behind the modal.
    await menu("close_project");
    await menu("new_project");
    await menu("command_palette");
    // Closing a project is asynchronous (open documents may save first); let it settle.
    await new Promise((resolve) => setTimeout(resolve, 0));
    await tick();
    expect(currentProject.value?.id).toBe(mockProject.id);
    expect(screen.queryByText("New project")).toBeNull();
    expect(screen.queryByPlaceholderText("Type a command or search…")).toBeNull();

    if (modal === "an app dialog")
      await fireEvent.keyDown(document.activeElement!, { key: "Escape" });
    else if (modal === "a file picker") {
      finishPicker();
      await tick();
    } else {
      ui.finishImport();
      await tick();
    }
    await menu("close_project");
    await waitFor(() => expect(currentProject.value).toBeNull());
  }
);

it("ignores shortcuts while a dialog is open even when focus has fallen to the page", async () => {
  render(App);
  await menu("about");
  await waitFor(() =>
    expect(screen.getByTestId("about-dialog").contains(document.activeElement)).toBe(true)
  );
  (document.activeElement as HTMLElement).blur();
  expect(document.activeElement).toBe(document.body);
  await fireEvent.keyDown(document.body, { key: "k", metaKey: true });
  await fireEvent.keyDown(document.body, { key: "k", ctrlKey: true });
  expect(screen.queryByPlaceholderText("Type a command or search…")).toBeNull();
  expect(screen.getByTestId("about-dialog")).toBeTruthy();
});

it("opens one settings window from the menu and reserves native project commands until it closes", async () => {
  render(App);
  await menu("settings");
  expect(await screen.findByRole("dialog", { name: "Settings" })).toBeTruthy();
  await menu("settings");
  expect(screen.getAllByRole("dialog", { name: "Settings" })).toHaveLength(1);
  await menu("new_project");
  expect(screen.queryByText("Create New Project")).toBeNull();
  await menu("close_project");
  expect(currentProject.value?.id).toBe(mockProject.id);
});

it.each(["projects", "editorial"])(
  "waits for both initial content requests when %s finishes first",
  async (first) => {
    currentProject.setProject(null);
    let projectsReady!: () => void;
    let editorialReady!: () => void;
    const projects = new Promise<never[]>((resolve) => (projectsReady = () => resolve([])));
    const editorial = new Promise<never[]>((resolve) => (editorialReady = () => resolve([])));
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "get_recent_projects") return projects;
      if (command === "take_editorial_open_files") return editorial;
      return [];
    });
    const onReady = vi.fn();
    render(App, { onReady });
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("take_editorial_open_files"));
    expect(onReady).not.toHaveBeenCalled();
    (first === "projects" ? projectsReady : editorialReady)();
    await tick();
    expect(onReady).not.toHaveBeenCalled();
    (first === "projects" ? editorialReady : projectsReady)();
    await waitFor(() => expect(onReady).toHaveBeenCalledOnce());
  }
);

it("settles startup even if initial content requests fail", async () => {
  currentProject.setProject(null);
  vi.spyOn(console, "error").mockImplementation(() => {});
  vi.mocked(invoke).mockRejectedValue(new Error("Database unavailable"));
  const onReady = vi.fn();
  render(App, { onReady });
  await waitFor(() => expect(onReady).toHaveBeenCalledOnce());
  expect(screen.getByText("Your campaigns will appear here")).toBeTruthy();
});

it("waits for a native file drain that starts before the listener handshake finishes", async () => {
  currentProject.setProject(null);
  let finish!: () => void;
  const pending = new Promise<never[]>((resolve) => (finish = () => resolve([])));
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === "take_editorial_open_files" ? pending : []
  );
  vi.mocked(listen).mockImplementation(async (event, callback) => {
    if (event === "editorial-open") callback({ event, id: 1, payload: undefined });
    return () => {};
  });
  try {
    const onReady = vi.fn();
    render(App, { onReady });
    await tick();
    expect(invoke).toHaveBeenCalledWith("take_editorial_open_files");
    expect(onReady).not.toHaveBeenCalled();
    finish();
    await waitFor(() => expect(onReady).toHaveBeenCalledOnce());
  } finally {
    vi.mocked(listen).mockResolvedValue(() => {});
  }
});

it("dispatches remapped commands once, drops old bindings, and ignores extra modifiers", async () => {
  currentProject.setProject(null);
  render(App);
  await tick();
  shortcuts.bindings.command_palette = "Mod+Alt+P";
  await fireEvent.keyDown(window, { key: "k", ctrlKey: true });
  expect(screen.queryByPlaceholderText("Type a command or search…")).toBeNull();
  await fireEvent.keyDown(window, { key: "p", ctrlKey: true, altKey: true, shiftKey: true });
  expect(screen.queryByPlaceholderText("Type a command or search…")).toBeNull();
  await fireEvent.keyDown(window, { key: "p", ctrlKey: true, altKey: true });
  expect(screen.getByPlaceholderText("Type a command or search…")).toBeTruthy();
});

it("blocks project commands and recording keys in Settings but keeps explicit Quit available", async () => {
  render(App);
  await menu("settings");
  await fireEvent.click(screen.getByRole("button", { name: "Keyboard Shortcuts" }));
  await menu("new_project");
  shortcuts.recording = true;
  await fireEvent.keyDown(window, { key: "q", ctrlKey: true });
  expect(exit).not.toHaveBeenCalled();
  expect(screen.getByTestId("settings-dialog")).toBeTruthy();
  expect(invoke).toHaveBeenCalledWith("suspend_keyboard_shortcuts", { suspended: true });
  await menu("quit");
  await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
});

it("preserves paste-as-plain-text and allows Quit from Settings when not recording", async () => {
  render(App);
  const event = new KeyboardEvent("keydown", {
    key: "V",
    ctrlKey: true,
    shiftKey: true,
    bubbles: true,
    cancelable: true,
  });
  window.dispatchEvent(event);
  expect(event.defaultPrevented).toBe(false);
  await menu("settings");
  await fireEvent.keyDown(window, { key: "q", ctrlKey: true });
  await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
});

function holdCampaignSave(result: Promise<boolean>) {
  const listener = (event: Event) => {
    (event as CustomEvent<{ pendingDocumentSaves: Promise<boolean>[] }>).detail.pendingDocumentSaves.push(
      result
    );
  };
  window.addEventListener("catalyst:flush-documents-before-close", listener);
  return () => window.removeEventListener("catalyst:flush-documents-before-close", listener);
}

it.each(["menu", "native"])(
  "waits for open campaign documents to save before %s quit",
  async (path) => {
    render(App);
    await tick();
    let finish!: (saved: boolean) => void;
    const release = holdCampaignSave(new Promise<boolean>((resolve) => (finish = resolve)));
    try {
      let closed = false;
      const event = { preventDefault: vi.fn() };
      const closing =
        path === "menu"
          ? menu("quit").then(() => (closed = true))
          : Promise.resolve(
              vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0](event as never)
            ).then(() => (closed = true));
      await tick();
      expect(exit).not.toHaveBeenCalled();
      if (path === "native") expect(closed).toBe(false);
      finish(true);
      await closing;
      if (path === "menu") await waitFor(() => expect(exit).toHaveBeenCalledWith(0));
      else expect(event.preventDefault).not.toHaveBeenCalled();
    } finally {
      release();
    }
  }
);

it.each(["menu", "native"])(
  "keeps Catalyst open through %s quit when a campaign document cannot save",
  async (path) => {
    render(App);
    await tick();
    const release = holdCampaignSave(Promise.resolve(false));
    try {
      const event = { preventDefault: vi.fn() };
      if (path === "menu") await menu("quit");
      else
        await vi.mocked(getCurrentWindow().onCloseRequested).mock.calls[0][0](event as never);
      await waitFor(() =>
        expect(ui.toast?.message).toContain("Some documents could not be saved")
      );
      expect(exit).not.toHaveBeenCalled();
      if (path === "native") expect(event.preventDefault).toHaveBeenCalled();
    } finally {
      release();
    }
  }
);
