#!/usr/bin/env node
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  normalizeUpdaterPathPrefix,
  normalizeUpdaterPublicUrl,
} from "./prepare-tauri-updater-config.mjs";

const expectedPlatforms = [
  "windows-x86_64",
  "darwin-aarch64",
  "darwin-x86_64",
  "linux-x86_64",
];

function walkFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...walkFiles(fullPath));
    else files.push(fullPath);
  }
  return files;
}

export function mergeUpdaterManifests({
  manifests,
  version,
  tag,
  publicUrl,
  pathPrefix = "aqardesk",
  files,
}) {
  if (manifests.length !== expectedPlatforms.length) {
    throw new Error(`Expected ${expectedPlatforms.length} updater manifests; found ${manifests.length}.`);
  }
  if (!/^v\d+\.\d+\.\d+$/.test(tag ?? "") || tag !== `v${version}`) {
    throw new Error("Invalid release tag or version.");
  }
  const updaterUrl = normalizeUpdaterPublicUrl(publicUrl);
  const prefix = normalizeUpdaterPathPrefix(pathPrefix);
  const artifactPatterns = {
    "windows-x86_64": /-setup\.exe$/,
    "darwin-aarch64": /\.app\.tar\.gz$/,
    "darwin-x86_64": /\.app\.tar\.gz$/,
    "linux-x86_64": /\.AppImage$/,
  };
  const merged = { ...manifests[0], platforms: {} };
  for (const manifest of manifests) {
    if (manifest.version !== version) throw new Error(`Updater version ${manifest.version} does not match ${version}.`);
    if (typeof manifest.notes !== "string" || !manifest.notes.trim()) throw new Error("Updater manifest is missing release notes.");
    if (manifest.notes.replace(/\r\n?/g, "\n").trim() !== manifests[0].notes.replace(/\r\n?/g, "\n").trim()) {
      throw new Error("Platform updater manifests contain different release notes.");
    }
    for (const [platform, entry] of Object.entries(manifest.platforms ?? {})) {
      if (!expectedPlatforms.includes(platform) || merged.platforms[platform]) {
        throw new Error(`Unexpected or duplicate updater platform: ${platform}.`);
      }
      if (
        typeof entry?.url !== "string" ||
        typeof entry?.signature !== "string" ||
        !entry.signature.trim()
      ) {
        throw new Error(`Updater entry is incomplete for ${platform}.`);
      }
      const url = new URL(entry.url);
      const encodedFilename = url.pathname.split("/").pop();
      const filename = decodeURIComponent(encodedFilename);
      if (
        entry.url !== `${updaterUrl}/${prefix}/releases/${tag}/${encodeURIComponent(filename)}` ||
        !artifactPatterns[platform].test(filename) ||
        !files.has(filename)
      ) {
        throw new Error(`Updater URL is invalid or references a missing artifact for ${platform}.`);
      }
      if (!files.has(`${filename}.sig`)) throw new Error(`Updater signature file is missing for ${platform}.`);
      entry.signature = entry.signature.trim();
      merged.platforms[platform] = entry;
    }
  }

  if (Object.keys(merged.platforms).sort().join(",") !== expectedPlatforms.slice().sort().join(",")) {
    throw new Error(`Updater platforms do not match expected targets: ${Object.keys(merged.platforms).join(", ")}.`);
  }
  const expectedFiles = new Set(
    Object.values(merged.platforms).flatMap((entry) => {
      const filename = decodeURIComponent(new URL(entry.url).pathname.split("/").pop());
      return [filename, `${filename}.sig`];
    }),
  );
  if ([...files].some((file) => !expectedFiles.has(file)) || expectedFiles.size !== files.size) {
    throw new Error("Downloaded release assets do not match the signed updater bundles.");
  }
  merged.notes = merged.notes.replace(/\r\n?/g, "\n").trim();
  return merged;
}

function run() {
  const tag = process.env.RELEASE_TAG;
  const version = process.env.RELEASE_VERSION;
  if (!/^v\d+\.\d+\.\d+$/.test(tag ?? "") || tag !== `v${version}`) {
    throw new Error(`Invalid release tag/version pair: ${tag} / ${version}.`);
  }
  const publicUrl = process.env.TAURI_UPDATER_PUBLIC_URL;
  const pathPrefix = process.env.TAURI_UPDATER_PATH_PREFIX ?? "aqardesk";
  const releaseRoot = path.resolve("release-artifacts");
  if (!statSync(releaseRoot, { throwIfNoEntry: false })?.isDirectory()) {
    throw new Error("Downloaded release artifacts are missing.");
  }
  const allFiles = walkFiles(releaseRoot);
  const manifestPaths = allFiles.filter((file) => path.basename(file) === "latest.json");
  if (manifestPaths.length !== expectedPlatforms.length) {
    throw new Error(`Expected ${expectedPlatforms.length} latest.json files; found ${manifestPaths.length}.`);
  }
  const manifests = manifestPaths.map((file) => JSON.parse(readFileSync(file, "utf8")));
  const assetNames = new Set(allFiles.filter((file) => !manifestPaths.includes(file)).map((file) => path.basename(file)));
  for (const manifest of manifests) {
    for (const entry of Object.values(manifest.platforms ?? {})) {
      const filename = decodeURIComponent(new URL(entry.url).pathname.split("/").pop());
      const signaturePath = allFiles.find((file) => path.basename(file) === `${filename}.sig`);
      if (!signaturePath) throw new Error(`Updater signature file is missing for ${filename}.`);
      const actualSignature = readFileSync(signaturePath, "utf8").trim();
      if (actualSignature !== String(entry.signature).trim()) {
        throw new Error(`Updater signature does not match ${filename}.`);
      }
    }
  }
  const merged = mergeUpdaterManifests({
    manifests,
    version,
    tag,
    publicUrl,
    pathPrefix,
    files: assetNames,
  });
  const packageDir = path.resolve("release-package");
  if (statSync(packageDir, { throwIfNoEntry: false })) {
    throw new Error("release-package already exists. Remove the previous generated package before retrying.");
  }
  mkdirSync(packageDir, { recursive: true });

  for (const platform of expectedPlatforms) {
    const filename = decodeURIComponent(new URL(merged.platforms[platform].url).pathname.split("/").pop());
    for (const asset of [filename, `${filename}.sig`]) {
      const source = allFiles.find((file) => path.basename(file) === asset);
      if (!source) throw new Error(`Release asset not found: ${asset}.`);
      copyFileSync(source, path.join(packageDir, asset));
    }
  }
  writeFileSync(path.join(packageDir, "latest.json"), `${JSON.stringify(merged, null, 2)}\n`, { flag: "wx" });
  console.log("Validated and merged updater metadata and signed release assets.");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    run();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
