#!/usr/bin/env node
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const targets = {
  "x86_64-pc-windows-msvc": {
    platform: "windows-x86_64",
    pattern: /-setup\.exe$/,
  },
  "aarch64-apple-darwin": {
    platform: "darwin-aarch64",
    pattern: /\.app\.tar\.gz$/,
  },
};

export function createUpdaterManifest({ repository, tag, version, target, artifactName, signature, notes }) {
  const targetConfig = targets[target];
  if (!targetConfig) throw new Error(`Unsupported updater target: ${target ?? "(missing)"}.`);
  if (!/^[^/]+\/[^/]+$/.test(repository ?? "")) throw new Error(`Invalid GitHub repository: ${repository}.`);
  if (!/^v\d+\.\d+\.\d+$/.test(tag ?? "") || tag !== `v${version}`) {
    throw new Error(`Release tag ${tag} does not match version ${version}.`);
  }
  if (
    typeof artifactName !== "string" ||
    !targetConfig.pattern.test(artifactName) ||
    /[\\/]/.test(artifactName)
  ) {
    throw new Error(`Invalid updater artifact name for ${targetConfig.platform}.`);
  }
  if (typeof signature !== "string" || !signature.trim()) throw new Error(`Missing signature for ${artifactName}.`);
  if (typeof notes !== "string" || !notes.trim()) throw new Error(`Missing release notes for ${tag}.`);

  return {
    version,
    notes: notes.replace(/\r\n?/g, "\n").trim(),
    platforms: {
      [targetConfig.platform]: {
        url: `https://github.com/${repository}/releases/download/${tag}/${encodeURIComponent(artifactName)}`,
        signature: signature.trim(),
      },
    },
  };
}

function run() {
  const target = process.env.TAURI_TARGET;
  const targetConfig = targets[target];
  if (!targetConfig) throw new Error(`Unsupported updater target: ${target ?? "(missing)"}.`);
  const artifactDir = path.resolve(process.env.CI_ARTIFACT_DIR ?? "ci-artifacts");
  const artifacts = readdirSync(artifactDir).filter((name) => targetConfig.pattern.test(name));
  if (artifacts.length !== 1) {
    throw new Error(`Expected one ${targetConfig.platform} bundle in ${artifactDir}; found ${artifacts.length}.`);
  }
  const [artifactName] = artifacts;
  const signaturePath = path.join(artifactDir, `${artifactName}.sig`);
  if (!statSync(signaturePath, { throwIfNoEntry: false })?.isFile()) {
    throw new Error(`Missing signature for ${artifactName}.`);
  }
  const signature = readFileSync(signaturePath, "utf8");
  const tag = process.env.RELEASE_TAG;
  const manifest = createUpdaterManifest({
    repository: process.env.GITHUB_REPOSITORY,
    tag,
    version: process.env.RELEASE_VERSION,
    target,
    artifactName,
    signature,
    notes: readFileSync(path.resolve(`releases/${tag}.md`), "utf8"),
  });
  writeFileSync(path.join(artifactDir, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`, { flag: "wx" });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    run();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

export { targets };
