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
import { DIE_SIDES, type DiceExpression, type DieResult } from "../types/dice";

/** A renderer that settles every die on its highest face. */
function maxRenderer() {
  const created: DiceRenderer[] = [];
  const factory = () => {
    const renderer: DiceRenderer = {
      kind: "physics",
      init: vi.fn(async () => {}),
      roll: vi.fn(async (expression: DiceExpression) =>
        expression.terms.flatMap((term) =>
          Array.from({ length: term.count }, (): DieResult => ({ type: term.type, value: DIE_SIDES[term.type] }))
        )
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

async function renderReady(factory: () => DiceRenderer) {
  diceTray.open("expanded");
  const view = render(DiceTray, { props: { createRenderer: factory } });
  await waitFor(() => expect(screen.getByTestId("dice-stage").dataset.state).toBe("ready"));
  return view;
}

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
  diceTray.endCampaignSession();
  diceTray.resetSelection();
  diceTray.visibility = "public";
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
      screen
        .getAllByTestId("dice-value")
        .map((node) => node.querySelector(".dice-chip__value")?.textContent),
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

  it("builds mixed rolls from the die buttons and + and −, and shows each die", async () => {
    const { factory } = maxRenderer();
    await renderReady(factory);
    await fireEvent.click(screen.getByTestId("dice-type-d6"));
    await fireEvent.click(screen.getByTestId("dice-add"));
    await fireEvent.click(screen.getByTestId("dice-count-modifier"));
    await fireEvent.click(screen.getByTestId("dice-add"));
    await fireEvent.click(screen.getByTestId("dice-add"));
    expect(diceTray.selection).toBe("2d20+2d6+2");
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => expect(screen.getByTestId("dice-total").textContent).toContain("54"));
    expect(screen.getByTestId("dice-expression").textContent).toBe("2d20+2d6+2");
    expect(screen.getAllByTestId("dice-value")).toHaveLength(4);
    expect(screen.getByTestId("dice-modifier").textContent).toContain("+2");
  });

  it("rolls advantage, marks the dropped die and shows a natural 20", async () => {
    const { factory } = maxRenderer();
    await renderReady(factory);
    diceTray.counts = { d20: 1 };
    await fireEvent.click(screen.getByTestId("dice-adv"));
    expect(screen.getByTestId("dice-adv").getAttribute("aria-pressed")).toBe("true");
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    expect(screen.getByTestId("dice-kind").textContent).toMatch(/adv/i);
    expect(screen.getByTestId("dice-critical").textContent).toBe("Critical success");
    const dropped = screen.getAllByTestId("dice-value").filter((node) => node.dataset.dropped === "true");
    expect(dropped).toHaveLength(1);
    expect(screen.getByTestId("dice-total").textContent).toContain("20");
  });

  it("rolls a typed expression and rejects an invalid one without rolling", async () => {
    const { factory, created } = maxRenderer();
    await renderReady(factory);
    await fireEvent.click(screen.getByTestId("dice-expression-toggle"));
    const input = screen.getByTestId("dice-expression-input") as HTMLInputElement;
    await fireEvent.input(input, { target: { value: "2d7" } });
    await fireEvent.click(screen.getByTestId("dice-expression-roll"));
    expect(screen.getByRole("alert").textContent).toMatch(/Unsupported die/);
    expect(created[0].roll).not.toHaveBeenCalled();

    await fireEvent.input(input, { target: { value: "3d8+4" } });
    await fireEvent.click(screen.getByTestId("dice-expression-roll"));
    await waitFor(() => expect(screen.getByTestId("dice-total").textContent).toContain("28"));
    expect(screen.getByTestId("dice-expression").textContent).toBe("3d8+4");
  });

  it("restarts the 3D table with new appearance settings and keeps history", async () => {
    const { factory, created } = maxRenderer();
    await renderReady(factory);
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    await fireEvent.click(screen.getByTestId("dice-settings-toggle"));
    await fireEvent.click(screen.getByTestId("dice-color-sapphire"));
    await waitFor(() => expect(created).toHaveLength(2));
    expect(created[0].dispose).toHaveBeenCalled();
    expect(diceTray.settings.diceColor).toBe("sapphire");
    expect(diceTray.history.all).toHaveLength(1);
    diceTray.updateSettings({ diceColor: "amber" });
  });

  it("prints the Catalyst logo into the mat as decoration only", async () => {
    const { factory } = maxRenderer();
    await renderReady(factory);
    const logo = document.querySelector(".dice-tray__mat-logo") as HTMLImageElement;
    expect(logo).not.toBeNull();
    expect(logo.getAttribute("alt")).toBe("");
    expect(logo.getAttribute("aria-hidden")).toBe("true");
    // Behind the dice canvas: it comes before the stage in the table.
    expect(logo.compareDocumentPosition(screen.getByTestId("dice-stage")) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it("keeps the result and history when the app re-renders around the tray", async () => {
    diceTray.open("compact");
    const first = render(DiceTray);
    await fireEvent.click(screen.getByTestId("dice-roll"));
    await waitFor(() => screen.getByTestId("dice-total"));
    const total = screen.getByTestId("dice-total").textContent;
    // Moving between workspace views remounts the tray while it stays open.
    first.unmount();
    render(DiceTray);
    expect(screen.getByTestId("dice-total").textContent).toBe(total);
    expect(diceTray.history.all).toHaveLength(1);
  });
});
