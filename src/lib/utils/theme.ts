export type ThemePreference = "dark" | "light" | "system";
/** Catalyst has a single primary accent, Frost Blue (design doc 025). */
export type AccentPreference = "blue";

const STORAGE_KEY = "lithomark:theme";
const LEGACY_STORAGE_KEY = "kindling:theme";
const ACCENT_STORAGE_KEY = "lithomark:accent";
let mediaQuery: MediaQueryList | null = null;
let mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

function resolveTheme(pref: ThemePreference): "dark" | "light" {
  if (pref === "system") {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return pref;
}

/**
 * Class applied to <html> for the duration of a theme switch. app.css disables
 * transitions under it so every surface flips at once instead of each element
 * animating from its old colour, and the forced style flush below makes WebKit
 * recompute var()-driven colours on the side panels immediately.
 */
const SWITCHING_CLASS = "theme-switching";

function applyTheme(theme: "dark" | "light") {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.classList?.add(SWITCHING_CLASS);
  root.setAttribute("data-theme", theme);
  // Reading a layout property forces a synchronous style recalculation.
  void root.offsetHeight;
  const release = () => root.classList?.remove(SWITCHING_CLASS);
  if (typeof requestAnimationFrame === "function") requestAnimationFrame(release);
  else release();
}

export function getStoredPreference(): ThemePreference {
  if (typeof localStorage === "undefined") return "dark";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") return stored;

  const legacy = localStorage.getItem(LEGACY_STORAGE_KEY);
  if (legacy === "light" || legacy === "dark" || legacy === "system") {
    localStorage.setItem(STORAGE_KEY, legacy);
    localStorage.removeItem(LEGACY_STORAGE_KEY);
    return legacy;
  }

  return "dark";
}

export function setThemePreference(pref: ThemePreference) {
  if (typeof localStorage !== "undefined") localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(resolveTheme(pref));
  setupSystemListener(pref);
}

export function getStoredAccent(): AccentPreference {
  // Older builds could store gold/teal/purple; those accents are retired, so a
  // stored value is ignored and Frost Blue always applies.
  return "blue";
}

export function setAccentPreference(pref: AccentPreference) {
  if (typeof localStorage !== "undefined") localStorage.setItem(ACCENT_STORAGE_KEY, pref);
  if (typeof document !== "undefined") document.documentElement.setAttribute("data-accent", pref);
}

function setupSystemListener(pref: ThemePreference) {
  if (mediaQuery && mediaListener) {
    mediaQuery.removeEventListener("change", mediaListener);
    mediaListener = null;
  }

  if (pref === "system" && typeof window !== "undefined" && typeof window.matchMedia === "function") {
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    mediaListener = (e: MediaQueryListEvent) => {
      applyTheme(e.matches ? "dark" : "light");
    };
    mediaQuery.addEventListener("change", mediaListener);
  }
}

/**
 * Call once on app startup (before first paint if possible).
 * Reads stored preference and applies the correct data-theme attribute.
 */
export function initTheme() {
  const pref = getStoredPreference();
  applyTheme(resolveTheme(pref));
  setupSystemListener(pref);
  if (typeof document !== "undefined") document.documentElement.setAttribute("data-accent", getStoredAccent());
}
