import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { createUpdaterManifest } from "./create-tauri-updater-manifest.mjs";
import { mergeUpdaterManifests } from "./merge-release-artifacts.mjs";
import {
  normalizeUpdaterPathPrefix,
  normalizeUpdaterPublicKey,
  normalizeUpdaterPublicUrl,
} from "./prepare-tauri-updater-config.mjs";
import { stage } from "./stage-release-artifacts.mjs";
import {
  createR2UploadPlan,
  planR2ReleasePrune,
} from "./upload-updater-to-r2.mjs";
import { validateLicenseBuildConfig } from "./validate-license-build-config.mjs";

const base = {
  publicUrl: "https://updates.example.com",
  tag: "v1.2.3",
  version: "1.2.3",
  notes: "# AqarDesk Desktop v1.2.3\n\n## Summary\n\nRelease.",
};

test("creates platform updater metadata with versioned public R2 URLs", () => {
  const manifest = createUpdaterManifest({
    ...base,
    target: "x86_64-pc-windows-msvc",
    artifactName: "AqarDesk_1.2.3_x64-setup.exe",
    signature: " signed-signature \n",
  });
  assert.deepEqual(manifest.platforms["windows-x86_64"], {
    url: "https://updates.example.com/aqardesk/releases/v1.2.3/AqarDesk_1.2.3_x64-setup.exe",
    signature: "signed-signature",
  });

  const linux = createUpdaterManifest({
    ...base,
    target: "x86_64-unknown-linux-gnu",
    artifactName: "AqarDesk_1.2.3_amd64.AppImage.tar.gz",
    signature: "linux-signature",
  });
  assert.deepEqual(linux.platforms["linux-x86_64"], {
    url: "https://updates.example.com/aqardesk/releases/v1.2.3/AqarDesk_1.2.3_amd64.AppImage.tar.gz",
    signature: "linux-signature",
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

test("merges complete Windows, macOS, and Linux updater metadata", () => {
  const windows = createUpdaterManifest({
    ...base,
    target: "x86_64-pc-windows-msvc",
    artifactName: "AqarDesk_1.2.3_x64-setup.exe",
    signature: "windows-signature",
  });
  const mac = createUpdaterManifest({
    ...base,
    target: "aarch64-apple-darwin",
    artifactName: "AqarDesk-aarch64.app.tar.gz",
    signature: "mac-arm-signature",
  });
  const macIntel = createUpdaterManifest({
    ...base,
    target: "x86_64-apple-darwin",
    artifactName: "AqarDesk-x64.app.tar.gz",
    signature: "mac-intel-signature",
  });
  const linux = createUpdaterManifest({
    ...base,
    target: "x86_64-unknown-linux-gnu",
    artifactName: "AqarDesk_1.2.3_amd64.AppImage.tar.gz",
    signature: "linux-signature",
  });
  const merged = mergeUpdaterManifests({
    manifests: [windows, mac, macIntel, linux],
    version: base.version,
    tag: base.tag,
    publicUrl: base.publicUrl,
    files: new Set([
      "AqarDesk_1.2.3_x64-setup.exe",
      "AqarDesk_1.2.3_x64-setup.exe.sig",
      "AqarDesk-aarch64.app.tar.gz",
      "AqarDesk-aarch64.app.tar.gz.sig",
      "AqarDesk-x64.app.tar.gz",
      "AqarDesk-x64.app.tar.gz.sig",
      "AqarDesk_1.2.3_amd64.AppImage.tar.gz",
      "AqarDesk_1.2.3_amd64.AppImage.tar.gz.sig",
    ]),
  });
  assert.deepEqual(Object.keys(merged.platforms).sort(), [
    "darwin-aarch64",
    "darwin-x86_64",
    "linux-x86_64",
    "windows-x86_64",
  ]);
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
      manifests: [windows, windows, windows, windows],
      version: base.version,
      tag: base.tag,
      publicUrl: base.publicUrl,
      files: new Set(["AqarDesk_1.2.3_x64-setup.exe", "AqarDesk_1.2.3_x64-setup.exe.sig"]),
    }),
    /duplicate updater platform/,
  );
});

test("validates the public updater key before generating build configuration", () => {
  const key = Buffer.alloc(42, 1).toString("base64");
  const minisignKey = `untrusted comment: release public key\n${key}`;
  const tauriKey = Buffer.from(minisignKey).toString("base64");
  assert.equal(
    normalizeUpdaterPublicKey(tauriKey),
    tauriKey,
  );
  assert.equal(
    normalizeUpdaterPublicKey(minisignKey),
    tauriKey,
  );
  assert.throws(() => normalizeUpdaterPublicKey(Buffer.from(tauriKey).toString("base64")), /valid minisign public key/);
  assert.throws(() => normalizeUpdaterPublicKey("not-a-minisign-key"), /Tauri-generated \.pub key/);
});

test("requires a public HTTPS origin for updater downloads", () => {
  assert.equal(normalizeUpdaterPublicUrl("https://updates.example.com/"), "https://updates.example.com");
  assert.throws(() => normalizeUpdaterPublicUrl("http://updates.example.com"), /HTTPS/);
  assert.throws(() => normalizeUpdaterPublicUrl("https://updates.example.com/updates"), /without a path/);
});

test("isolates updater files with a safe shared-bucket prefix", () => {
  assert.equal(normalizeUpdaterPathPrefix(" aqardesk "), "aqardesk");
  assert.equal(normalizeUpdaterPathPrefix("apps/aqardesk"), "apps/aqardesk");
  assert.throws(() => normalizeUpdaterPathPrefix("../orbis"), /lowercase letters/);
});

test("plans versioned R2 bundle uploads before publishing the latest manifest", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "aqardesk-r2-release-"));
  const windowsName = "AqarDesk_1.2.3_x64-setup.exe";
  const macName = "AqarDesk-aarch64.app.tar.gz";
  const macIntelName = "AqarDesk-x64.app.tar.gz";
  const linuxName = "AqarDesk_1.2.3_amd64.AppImage.tar.gz";
  const windows = createUpdaterManifest({
    ...base,
    target: "x86_64-pc-windows-msvc",
    artifactName: windowsName,
    signature: "windows-signature",
  });
  const mac = createUpdaterManifest({
    ...base,
    target: "aarch64-apple-darwin",
    artifactName: macName,
    signature: "mac-signature",
  });
  const macIntel = createUpdaterManifest({
    ...base,
    target: "x86_64-apple-darwin",
    artifactName: macIntelName,
    signature: "mac-intel-signature",
  });
  const linux = createUpdaterManifest({
    ...base,
    target: "x86_64-unknown-linux-gnu",
    artifactName: linuxName,
    signature: "linux-signature",
  });
  const manifest = mergeUpdaterManifests({
    manifests: [windows, mac, macIntel, linux],
    version: base.version,
    tag: base.tag,
    publicUrl: base.publicUrl,
    files: new Set([
      windowsName,
      `${windowsName}.sig`,
      macName,
      `${macName}.sig`,
      macIntelName,
      `${macIntelName}.sig`,
      linuxName,
      `${linuxName}.sig`,
    ]),
  });
  writeFileSync(path.join(root, windowsName), "windows bundle");
  writeFileSync(path.join(root, macName), "mac bundle");
  writeFileSync(path.join(root, macIntelName), "mac Intel bundle");
  writeFileSync(path.join(root, linuxName), "Linux bundle");
  writeFileSync(path.join(root, "latest.json"), JSON.stringify(manifest));

  try {
    const plan = createR2UploadPlan({
      publicUrl: base.publicUrl,
      accountId: "a".repeat(32),
      bucket: "aqardesk-updates",
      tag: base.tag,
      packageDir: root,
    });
    assert.equal(plan.length, 5);
    assert.deepEqual(
      plan.map((item) => item.destination),
      [
        `s3://aqardesk-updates/aqardesk/releases/${base.tag}/${windowsName}`,
        `s3://aqardesk-updates/aqardesk/releases/${base.tag}/${macName}`,
        `s3://aqardesk-updates/aqardesk/releases/${base.tag}/${macIntelName}`,
        `s3://aqardesk-updates/aqardesk/releases/${base.tag}/${linuxName}`,
        "s3://aqardesk-updates/aqardesk/latest.json",
      ],
    );
    assert.equal(plan.at(-1).cacheControl, "no-cache, no-store, must-revalidate");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("keeps the newest three AqarDesk releases and prunes only its own prefix", () => {
  const plan = planR2ReleasePrune({
    pathPrefix: "aqardesk",
    latestTag: "v1.0.10",
    keys: [
      "aqardesk/latest.json",
      "aqardesk/releases/v1.0.1/AqarDesk.exe",
      "aqardesk/releases/v1.0.9/AqarDesk.exe",
      "aqardesk/releases/v1.0.10/AqarDesk.exe",
      "aqardesk/releases/v1.0.2/AqarDesk.exe",
      "orbis/releases/v5.0.0/Orbis.exe",
    ],
  });
  assert.deepEqual(plan, [{ tag: "v1.0.1", prefix: "aqardesk/releases/v1.0.1/" }]);
});

test("refuses to prune unless the current release exists and is newest in R2", () => {
  assert.throws(
    () => planR2ReleasePrune({
      pathPrefix: "aqardesk",
      latestTag: "v1.0.3",
      keys: ["aqardesk/releases/v1.0.1/AqarDesk.exe"],
    }),
    /was not found in R2; refusing to prune/,
  );
  assert.throws(
    () => planR2ReleasePrune({
      pathPrefix: "aqardesk",
      latestTag: "v1.0.2",
      keys: [
        "aqardesk/releases/v1.0.2/AqarDesk.exe",
        "aqardesk/releases/v1.0.3/AqarDesk.exe",
      ],
    }),
    /not the newest version in R2; refusing to prune/,
  );
});

test("stages signed bundles from the Cargo workspace target directory", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "aqardesk-stage-release-"));
  const target = "x86_64-pc-windows-msvc";
  const bundleName = "AqarDesk_1.2.3_x64-setup.exe";
  const bundleDir = path.join(root, "target", target, "release", "bundle", "nsis");
  mkdirSync(bundleDir, { recursive: true });
  writeFileSync(path.join(bundleDir, bundleName), "bundle");
  writeFileSync(path.join(bundleDir, `${bundleName}.sig`), "signature");

  try {
    stage({ root, target, bundle: "nsis" });
    assert.equal(readFileSync(path.join(root, "ci-artifacts", bundleName), "utf8"), "bundle");
    assert.equal(readFileSync(path.join(root, "ci-artifacts", `${bundleName}.sig`), "utf8"), "signature");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("stages Linux AppImage updater bundles and signatures", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "aqardesk-stage-linux-release-"));
  const target = "x86_64-unknown-linux-gnu";
  const bundleName = "AqarDesk_1.2.3_amd64.AppImage.tar.gz";
  const bundleDir = path.join(root, "target", target, "release", "bundle", "appimage");
  mkdirSync(bundleDir, { recursive: true });
  writeFileSync(path.join(bundleDir, bundleName), "bundle");
  writeFileSync(path.join(bundleDir, `${bundleName}.sig`), "signature");

  try {
    stage({ root, target, bundle: "appimage" });
    assert.equal(readFileSync(path.join(root, "ci-artifacts", bundleName), "utf8"), "bundle");
    assert.equal(readFileSync(path.join(root, "ci-artifacts", `${bundleName}.sig`), "utf8"), "signature");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("stages macOS updater bundles with architecture-specific names", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "aqardesk-stage-macos-release-"));
  const bundleName = "AqarDesk.app.tar.gz";

  try {
    for (const [target, stagedName] of [
      ["aarch64-apple-darwin", "AqarDesk-aarch64.app.tar.gz"],
      ["x86_64-apple-darwin", "AqarDesk-x86_64.app.tar.gz"],
    ]) {
      const bundleDir = path.join(root, "target", target, "release", "bundle", "macos");
      mkdirSync(bundleDir, { recursive: true });
      writeFileSync(path.join(bundleDir, bundleName), "bundle");
      writeFileSync(path.join(bundleDir, `${bundleName}.sig`), "signature");

      stage({ root, target, bundle: "macos" });
      assert.equal(readFileSync(path.join(root, "ci-artifacts", stagedName), "utf8"), "bundle");
      assert.equal(readFileSync(path.join(root, "ci-artifacts", `${stagedName}.sig`), "utf8"), "signature");
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("validates the public Mulverse license configuration required by release builds", () => {
  const config = {
    MULVERSE_LICENSE_API_BASE_URL: "https://api.mulverse.com",
    MULVERSE_LICENSE_PRODUCT_CODE: "AQARDESK",
    MULVERSE_LICENSE_PRODUCT_ID: "123e4567-e89b-42d3-a456-426614174000",
    MULVERSE_LICENSE_KEY_PREFIX: "AQAR",
    MULVERSE_LICENSE_PUBLIC_KEY_ID: "aqar-prod-1",
    MULVERSE_LICENSE_PUBLIC_KEY_B64: Buffer.alloc(32, 1).toString("base64"),
  };
  assert.doesNotThrow(() => validateLicenseBuildConfig(config));
  assert.throws(
    () => validateLicenseBuildConfig({ ...config, MULVERSE_LICENSE_PUBLIC_KEY_B64: "" }),
    /MULVERSE_LICENSE_PUBLIC_KEY_B64/,
  );
  assert.throws(
    () => validateLicenseBuildConfig({ ...config, MULVERSE_LICENSE_API_BASE_URL: "http://api.mulverse.com" }),
    /HTTPS URL/,
  );
});
