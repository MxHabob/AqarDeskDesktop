#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  normalizeUpdaterPathPrefix,
  normalizeUpdaterPublicUrl,
} from "./prepare-tauri-updater-config.mjs";

const expectedPlatforms = ["windows-x86_64", "darwin-aarch64"];
const artifactPatterns = {
  "windows-x86_64": /-setup\.exe$/,
  "darwin-aarch64": /\.app\.tar\.gz$/,
};

export function createR2UploadPlan({
  publicUrl,
  pathPrefix = "aqardesk",
  accountId,
  bucket,
  tag,
  packageDir,
}) {
  const updaterUrl = normalizeUpdaterPublicUrl(publicUrl);
  const prefix = normalizeUpdaterPathPrefix(pathPrefix);
  if (!/^[a-f0-9]{32}$/i.test(accountId ?? "")) {
    throw new Error("CLOUDFLARE_R2_ACCOUNT_ID must be the 32-character Cloudflare account ID.");
  }
  if (!/^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$/.test(bucket ?? "")) {
    throw new Error("CLOUDFLARE_R2_BUCKET must be a valid R2 bucket name.");
  }
  if (!/^v\d+\.\d+\.\d+$/.test(tag ?? "")) {
    throw new Error(`Invalid release tag: ${tag ?? "(missing)"}.`);
  }

  const root = path.resolve(packageDir);
  const manifestPath = path.join(root, "latest.json");
  if (!statSync(manifestPath, { throwIfNoEntry: false })?.isFile()) {
    throw new Error(`Updater manifest is missing: ${manifestPath}.`);
  }
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (manifest.version !== tag.slice(1)) {
    throw new Error(`Updater manifest version ${manifest.version} does not match ${tag}.`);
  }

  const platformNames = Object.keys(manifest.platforms ?? {}).sort();
  if (platformNames.join(",") !== expectedPlatforms.slice().sort().join(",")) {
    throw new Error("Updater manifest does not contain exactly the Windows and macOS builds.");
  }

  const endpointUrl = `https://${accountId}.r2.cloudflarestorage.com`;
  const destination = `s3://${bucket}`;
  const plan = [];
  for (const platform of expectedPlatforms) {
    const entry = manifest.platforms[platform];
    if (typeof entry?.url !== "string") {
      throw new Error(`Updater URL is missing for ${platform}.`);
    }
    const filename = decodeURIComponent(new URL(entry.url).pathname.split("/").pop());
    if (
      !artifactPatterns[platform].test(filename) ||
      entry.url !== `${updaterUrl}/${prefix}/releases/${tag}/${encodeURIComponent(filename)}`
    ) {
      throw new Error(`Updater URL is invalid for ${platform}.`);
    }
    const source = path.join(root, filename);
    if (!statSync(source, { throwIfNoEntry: false })?.isFile()) {
      throw new Error(`Updater bundle is missing: ${source}.`);
    }
    plan.push({
      source,
      destination: `${destination}/${prefix}/releases/${tag}/${filename}`,
      endpointUrl,
      cacheControl: "public, max-age=31536000, immutable",
      contentType: "application/octet-stream",
    });
  }

  plan.push({
    source: manifestPath,
    destination: `${destination}/${prefix}/latest.json`,
    endpointUrl,
    cacheControl: "no-cache, no-store, must-revalidate",
    contentType: "application/json",
  });
  return plan;
}

function run() {
  if (!process.env.CLOUDFLARE_R2_ACCESS_KEY_ID || !process.env.CLOUDFLARE_R2_SECRET_ACCESS_KEY) {
    throw new Error("Set CLOUDFLARE_R2_ACCESS_KEY_ID and CLOUDFLARE_R2_SECRET_ACCESS_KEY in GitHub Actions secrets.");
  }
  const plan = createR2UploadPlan({
    publicUrl: process.env.TAURI_UPDATER_PUBLIC_URL,
    pathPrefix: process.env.TAURI_UPDATER_PATH_PREFIX ?? "aqardesk",
    accountId: process.env.CLOUDFLARE_R2_ACCOUNT_ID,
    bucket: process.env.CLOUDFLARE_R2_BUCKET,
    tag: process.env.RELEASE_TAG,
    packageDir: path.resolve("release-package"),
  });
  for (const upload of plan) {
    const result = spawnSync(
      "aws",
      [
        "s3",
        "cp",
        upload.source,
        upload.destination,
        "--endpoint-url",
        upload.endpointUrl,
        "--region",
        "auto",
        "--cache-control",
        upload.cacheControl,
        "--content-type",
        upload.contentType,
        "--only-show-errors",
      ],
      { stdio: "inherit" },
    );
    if (result.error) throw new Error(`Could not start AWS CLI: ${result.error.message}`);
    if (result.status !== 0) throw new Error(`R2 upload failed with exit code ${result.status}.`);
  }
  console.log("Uploaded versioned updater bundles and latest.json to R2.");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    run();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
