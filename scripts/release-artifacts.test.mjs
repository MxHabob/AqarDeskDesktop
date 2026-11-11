import assert from "node:assert/strict";
import test from "node:test";
import { createUpdaterManifest } from "./create-tauri-updater-manifest.mjs";
import { mergeUpdaterManifests } from "./merge-release-artifacts.mjs";
import { normalizeUpdaterPublicKey } from "./prepare-tauri-updater-config.mjs";

const base = {
  repository: "MxHabob/AqarDeskDesktop",
  tag: "v1.2.3",
  version: "1.2.3",
  notes: "# AqarDesk Desktop v1.2.3\n\n## Summary\n\nRelease.",
};

test("creates platform updater metadata with signed release URLs", () => {
  const manifest = createUpdaterManifest({
    ...base,
    target: "x86_64-pc-windows-msvc",
    artifactName: "AqarDesk_1.2.3_x64-setup.exe",
    signature: " signed-signature \n",
  });
  assert.deepEqual(manifest.platforms["windows-x86_64"], {
    url: "https://github.com/MxHabob/AqarDeskDesktop/releases/download/v1.2.3/AqarDesk_1.2.3_x64-setup.exe",
    signature: "signed-signature",
  });
});

test("rejects mismatched tags, unsupported targets, and invalid artifacts", () => {
  assert.throws(
    () => createUpdaterManifest({ ...base, tag: "v1.2.4", target: "aarch64-apple-darwin", artifactName: "AqarDesk.app.tar.gz", signature: "sig", notes: base.notes }),
    /does not match version/,
  );
  assert.throws(
    () => createUpdaterManifest({ ...base, target: "linux", artifactName: "bad", signature: "sig", notes: base.notes }),
    /Unsupported updater target/,
  );
  assert.throws(
    () => createUpdaterManifest({ ...base, target: "aarch64-apple-darwin", artifactName: "../AqarDesk.app.tar.gz", signature: "sig", notes: base.notes }),
    /Invalid updater artifact name/,
  );
});

test("merges complete Windows and macOS updater metadata", () => {
  const windows = createUpdaterManifest({
    ...base,
    target: "x86_64-pc-windows-msvc",
    artifactName: "AqarDesk_1.2.3_x64-setup.exe",
    signature: "windows-signature",
  });
  const mac = createUpdaterManifest({
    ...base,
    target: "aarch64-apple-darwin",
    artifactName: "AqarDesk.app.tar.gz",
    signature: "mac-signature",
  });
  const merged = mergeUpdaterManifests({
    manifests: [windows, mac],
    version: base.version,
    tag: base.tag,
    repository: base.repository,
    files: new Set([
      "AqarDesk_1.2.3_x64-setup.exe",
      "AqarDesk_1.2.3_x64-setup.exe.sig",
      "AqarDesk.app.tar.gz",
      "AqarDesk.app.tar.gz.sig",
    ]),
  });
  assert.deepEqual(Object.keys(merged.platforms).sort(), ["darwin-aarch64", "windows-x86_64"]);
});

test("rejects incomplete or conflicting updater metadata", () => {
  const windows = createUpdaterManifest({
    ...base,
    target: "x86_64-pc-windows-msvc",
    artifactName: "AqarDesk_1.2.3_x64-setup.exe",
    signature: "windows-signature",
  });
  assert.throws(
    () => mergeUpdaterManifests({
      manifests: [windows, windows],
      version: base.version,
      tag: base.tag,
      repository: base.repository,
      files: new Set(["AqarDesk_1.2.3_x64-setup.exe", "AqarDesk_1.2.3_x64-setup.exe.sig"]),
    }),
    /duplicate updater platform/,
  );
});

test("validates the public updater key before generating build configuration", () => {
  const key = Buffer.alloc(42, 1).toString("base64");
  assert.equal(
    normalizeUpdaterPublicKey(`untrusted comment: release public key\n${key}`),
    `untrusted comment: release public key\n${key}`,
  );
  assert.throws(() => normalizeUpdaterPublicKey("not-a-minisign-key"), /minisign/);
});
