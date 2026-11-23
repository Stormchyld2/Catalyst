// The bundled Dice Box assets must be the exact files from the pinned package,
// so the offline copy cannot drift from the code that loads it.
import { describe, expect, it } from "vitest";

// Node modules are imported indirectly, as in ScenePanel.test.ts, because the
// app tsconfig has no Node types.
interface NodeFs {
  readFileSync(
    path: string,
    encoding?: string,
  ): { equals(other: unknown): boolean } & string;
  readdirSync(path: string): string[];
  statSync(path: string): { isDirectory(): boolean };
}
const fsModule = "node:fs";
const pathModule = "node:path";
const fs: NodeFs = await import(/* @vite-ignore */ fsModule);
const { join, relative } = (await import(/* @vite-ignore */ pathModule)) as {
  join(...parts: string[]): string;
  relative(from: string, to: string): string;
};

const root = (
  globalThis as unknown as { process: { cwd(): string } }
).process.cwd();
const packaged = join(root, "node_modules/@3d-dice/dice-box/dist/assets");
const bundled = join(root, "static/assets/dice-box");

function files(dir: string): string[] {
  return fs.readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return fs.statSync(path).isDirectory() ? files(path) : [path];
  });
}

describe("bundled Dice Box assets", () => {
  it("match the installed @3d-dice/dice-box version byte for byte", () => {
    const expected = files(packaged)
      .map((path) => relative(packaged, path))
      .sort();
    expect(expected).toContain("ammo/ammo.wasm.wasm");
    expect(expected).toContain("themes/default/theme.config.json");
    for (const path of expected) {
      const same = fs
        .readFileSync(join(bundled, path))
        .equals(fs.readFileSync(join(packaged, path)));
      expect(same, path).toBe(true);
    }
  });

  it("ship the licence texts next to the assets", () => {
    expect(fs.readFileSync(join(bundled, "LICENSE"), "utf8")).toMatch(
      /MIT License/,
    );
    const notices = fs.readFileSync(
      join(bundled, "THIRD-PARTY-LICENSES.txt"),
      "utf8",
    );
    expect(notices).toMatch(/Apache License/);
    expect(notices).toMatch(/zlib/);
  });

  it("pin the package to an exact version", () => {
    const pkg = JSON.parse(fs.readFileSync(join(root, "package.json"), "utf8"));
    expect(pkg.dependencies["@3d-dice/dice-box"]).toMatch(/^\d+\.\d+\.\d+$/);
  });
});
