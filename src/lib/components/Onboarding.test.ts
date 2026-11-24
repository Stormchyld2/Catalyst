// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { ui } from "../stores/ui.svelte";
import Onboarding from "./Onboarding.svelte";

beforeEach(() => { ui.startOnboarding(); });
afterEach(() => { cleanup(); ui.skipOnboarding(); });

describe("Catalyst RPG onboarding", () => {
  it("introduces campaign workflow without novel imports", async () => {
    render(Onboarding);
    expect(screen.getByRole("dialog", { name: "Welcome to Catalyst" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText(/World Builder, Adventures, Characters and Campaign Notes/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByText(/The Hollow Crown/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    expect(screen.getByRole("dialog", { name: "Import RPG material" })).toBeTruthy();
    expect(screen.queryByText(/Scrivener|Plottr|novelWriter|yWriter/)).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Finish" }));
    expect(ui.showOnboarding).toBe(false);
  });

  it("skips safely without writing or importing anything", async () => {
    render(Onboarding);
    await fireEvent.click(screen.getByTestId("skip-onboarding"));
    expect(ui.showOnboarding).toBe(false);
  });

  it("supports Back and Skip for now", async () => {
    render(Onboarding);
    await fireEvent.click(screen.getByRole("button", { name: "Next" }));
    await fireEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(ui.onboardingStep).toBe("welcome");
    await fireEvent.click(screen.getByRole("button", { name: "Skip for now" }));
    expect(ui.showOnboarding).toBe(false);
  });
});
