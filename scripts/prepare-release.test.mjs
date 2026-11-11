import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import {
  bumpVersion,
  classifyCommit,
  compareVersions,
  parseVersion,
  renderReleaseNotes,
  updateCargoLockVersion,
  updateTomlPackageVersion,
} from "./prepare-release.mjs";

test("parses and compares stable SemVer versions", () => {
  assert.deepEqual(parseVersion("1.2.3"), [1, 2, 3]);
  assert.equal(compareVersions("1.10.0", "1.9.9"), 1);
  assert.throws(() => parseVersion("1.2"), /stable SemVer/);
});

test("bumps requested version level", () => {
  assert.equal(bumpVersion("1.2.3", "patch"), "1.2.4");
  assert.equal(bumpVersion("1.2.3", "minor"), "1.3.0");
  assert.equal(bumpVersion("1.2.3", "major"), "2.0.0");
  assert.throws(() => bumpVersion("1.2.3", "nightly"), /Unknown bump type/);
});

test("classifies conventional commits and breaking changes", () => {
  assert.deepEqual(classifyCommit({ subject: "feat(grid): export to CSV", body: "" }), {
    category: "feat",
    entry: "- export to CSV (grid)",
  });
  assert.equal(
    classifyCommit({ subject: "fix!: remove obsolete API", body: "" }).category,
    "breaking",
  );
});

test("generates editable, categorized AqarDesk release notes", () => {
  const notes = renderReleaseNotes("1.2.3", [
    { subject: "feat(search): keyboard navigation", body: "" },
    { subject: "fix: correct receipt total", body: "" },
  ]);
  assert.match(notes, /^# AqarDesk Desktop v1\.2\.3/m);
  assert.match(notes, /## Added[\s\S]*keyboard navigation \(search\)/);
  assert.match(notes, /## Fixed[\s\S]*correct receipt total/);
  assert.match(notes, /## Known Issues/);
});

test("updates only the desktop Cargo package version", () => {
  const manifest = '[package]\nname = "aqardesk-desktop"\nversion = "0.1.0"\n';
  const lock = '[[package]]\nname = "aqardesk-desktop"\nversion = "0.1.0"\n';
  assert.match(updateTomlPackageVersion(manifest, "1.0.1"), /version = "1\.0\.1"/);
  assert.match(updateCargoLockVersion(lock, "1.0.1"), /version = "1\.0\.1"/);
});

test("prepares a release in a clean repository and synchronizes all app versions", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "aqardesk-release-test-"));
  const write = (relativePath, contents) => {
    const destination = path.join(root, relativePath);
    mkdirSync(path.dirname(destination), { recursive: true });
    writeFileSync(destination, contents);
  };
  const runGit = (...args) => execFileSync("git", args, { cwd: root, stdio: "ignore" });

  try {
    write("scripts/prepare-release.mjs", readFileSync(
      fileURLToPath(new URL("./prepare-release.mjs", import.meta.url)),
    ));
    write("apps/desktop/package.json", JSON.stringify({ version: "1.0.0" }));
    write("apps/desktop/src-tauri/Cargo.toml", '[package]\nname = "aqardesk-desktop"\nversion = "1.0.0"\n');
    write("Cargo.lock", '[[package]]\nname = "aqardesk-desktop"\nversion = "1.0.0"\n');
    write("apps/desktop/src-tauri/tauri.conf.json", JSON.stringify({ version: "1.0.0" }));
    write("apps/desktop/src-tauri/tauri.dev.conf.json", JSON.stringify({ version: "1.0.0" }));
    runGit("init", "--quiet");
    runGit("config", "user.name", "Release Test");
    runGit("config", "user.email", "release-test@example.invalid");
    runGit("add", ".");
    runGit("commit", "--quiet", "-m", "feat: add release preparation");

    execFileSync(process.execPath, ["scripts/prepare-release.mjs", "patch"], { cwd: root });

    assert.equal(JSON.parse(readFileSync(path.join(root, "apps/desktop/package.json"), "utf8")).version, "1.0.1");
    assert.match(readFileSync(path.join(root, "apps/desktop/src-tauri/Cargo.toml"), "utf8"), /version = "1\.0\.1"/);
    assert.match(readFileSync(path.join(root, "Cargo.lock"), "utf8"), /version = "1\.0\.1"/);
    assert.equal(JSON.parse(readFileSync(path.join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8")).version, "1.0.1");
    assert.equal(JSON.parse(readFileSync(path.join(root, "apps/desktop/src-tauri/tauri.dev.conf.json"), "utf8")).version, "1.0.1");
    assert.match(readFileSync(path.join(root, "releases/v1.0.1.md"), "utf8"), /add release preparation/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
