// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { flushSync } from "svelte";
import DiceTray from "../components/DiceTray.svelte";
import { diceTray } from "../stores/diceTray.svelte";
import type { DiceRenderer } from "../engine/diceRenderer";
import type { DieResult } from "../types/dice";

function fakeRenderer(values: number[][]) {
  const created: DiceRenderer[] = [];
  const factory = () => {
    const queue = [...values];
    const renderer: DiceRenderer = {
      kind: "physics",
      init: vi.fn(async (container: HTMLElement) => {
        container.appendChild(document.createElement("canvas"));
      }),
      roll: vi.fn(async () =>
        (queue.shift() ?? [1, 1]).map(
          (value): DieResult => ({ type: "d20", value }),
        ),
      ),
      clear: vi.fn(),
      resize: vi.fn(),
      dispose: vi.fn(),
    };
    created.push(renderer);
    return renderer;
  };
  return { factory, created };
}

beforeEach(() => {
  diceTray.close();
  diceTray.history.clear();
  diceTray.clearTray();
  diceTray.counts = { d20: 2 };
  diceTray.selectedType = "d20";
  diceTray.updateSettings({ animate: true, lowPerformance: false });
});

afterEach(() => {
  cleanup();
  document.body.innerHTML = "";
});

describe("DiceTray", () => {
  it("is a nonblocking panel, not a modal", () => {
    diceTray.open("expanded");
    const { factory } = fakeRenderer([]);
    render(DiceTray, { props: { createRenderer: factory } });
    const tray = screen.getByTestId("dice-tray");
    expect(tray.getAttribute("aria-modal")).toBeNull();
    expect(tray.closest("[role=dialog]")).toBeNull();
    expect(document.querySelector(".dialog-scrim")).toBeNull();
  });

  it("rolls 2d20 with physics and shows each die and the total", async () => {
    diceTray.open("expanded");
    const { factory, created } = fakeRenderer([[14, 7]]);
    render(DiceTray, { props: { createRenderer: factory } });
    await waitFor(() =>
      expect(screen.getByTestId("dice-stage").dataset.state).toBe("ready"),
    );

    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() =>
      expect(screen.getByTestId("dice-total").textContent).toContain("21"),
    );
    expect(
      screen.getAllByTestId("dice-value").map((node) => node.textContent),
    ).toEqual(["14", "7"]);
    expect(created[0].roll).toHaveBeenCalledTimes(1);
    expect(diceTray.history.latest?.resolution).toBe("physics");
  });

  it("clears the tray but keeps history", async () => {
    diceTray.open("expanded");
    const { factory } = fakeRenderer([[3, 4]]);
    render(DiceTray, { props: { createRenderer: factory } });
    await waitFor(() =>
      expect(screen.getByTestId("dice-stage").dataset.state).toBe("ready"),
    );
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    await fireEvent.click(screen.getByTestId("dice-clear"));
    expect(screen.queryByTestId("dice-total")).toBeNull();
    expect(diceTray.history.all).toHaveLength(1);
  });

  it("disposes the renderer when switching to compact mode, and rolls instantly there", async () => {
    diceTray.open("expanded");
    const { factory, created } = fakeRenderer([]);
    render(DiceTray, { props: { createRenderer: factory } });
    await waitFor(() =>
      expect(screen.getByTestId("dice-stage").dataset.state).toBe("ready"),
    );

    await fireEvent.click(screen.getByTestId("dice-compact"));
    expect(created[0].dispose).toHaveBeenCalled();
    expect(screen.queryByTestId("dice-stage")).toBeNull();

    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    expect(diceTray.history.latest?.resolution).toBe("random");
    expect(created[0].roll).not.toHaveBeenCalled();
  });

  it("keeps history after closing and reopening", async () => {
    diceTray.open("compact");
    const first = render(DiceTray);
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    await fireEvent.click(screen.getByTestId("dice-close"));
    expect(diceTray.isOpen).toBe(false);
    first.unmount();

    diceTray.open("compact");
    render(DiceTray);
    await fireEvent.click(screen.getByTestId("dice-history-toggle"));
    expect(screen.getAllByTestId("dice-history-item")).toHaveLength(1);
  });

  it("falls back to instant rolls when 3D is unavailable", async () => {
    diceTray.open("expanded");
    const factory = (): DiceRenderer => ({
      kind: "physics",
      init: vi.fn(async () => {
        throw new Error("WebGL is not available on this device.");
      }),
      roll: vi.fn(),
      clear: vi.fn(),
      resize: vi.fn(),
      dispose: vi.fn(),
    });
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    render(DiceTray, { props: { createRenderer: factory } });
    await waitFor(() =>
      expect(screen.getByTestId("dice-stage").dataset.state).toBe(
        "unavailable",
      ),
    );
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    expect(diceTray.history.latest?.resolution).toBe("random");
    warn.mockRestore();
  });

  it("does not start the 3D renderer when animation is off", () => {
    diceTray.updateSettings({ animate: false });
    diceTray.open("expanded");
    const { factory, created } = fakeRenderer([]);
    render(DiceTray, { props: { createRenderer: factory } });
    expect(created).toHaveLength(0);
    expect(screen.queryByTestId("dice-stage")).toBeNull();
  });

  it("closes on Escape from inside the tray", async () => {
    diceTray.open("compact");
    render(DiceTray);
    await fireEvent.keyDown(screen.getByTestId("dice-roll"), { key: "Escape" });
    flushSync();
    expect(diceTray.isOpen).toBe(false);
  });

  it("moves with arrow keys on the grip and remembers the position", async () => {
    diceTray.open("compact");
    diceTray.moveTo(100, 100);
    render(DiceTray);
    await fireEvent.keyDown(screen.getByTestId("dice-grip"), {
      key: "ArrowRight",
    });
    expect(diceTray.activeGeometry.x).toBe(116);
    expect(
      JSON.parse(localStorage.getItem("catalyst:diceTray")!).compact.x,
    ).toBe(116);
  });
});
