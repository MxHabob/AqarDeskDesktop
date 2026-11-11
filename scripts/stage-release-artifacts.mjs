#!/usr/bin/env node
import { copyFileSync, mkdirSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const releaseTargets = {
  "x86_64-pc-windows-msvc": {
    bundle: "nsis",
    pattern: /-setup\.exe$/,
  },
  "aarch64-apple-darwin": {
    bundle: "macos",
    pattern: /\.app\.tar\.gz$/,
  },
};

function stage() {
  const target = process.env.TAURI_TARGET;
  const config = releaseTargets[target];
  if (!config) throw new Error(`Unsupported desktop release target: ${target ?? "(missing)"}.`);
  if (process.env.TAURI_BUNDLE !== config.bundle) {
    throw new Error(`Expected ${config.bundle} bundle directory for ${target}.`);
  }

  const bundleDir = path.resolve(
    `apps/desktop/src-tauri/target/${target}/release/bundle/${config.bundle}`,
  );
  if (!statSync(bundleDir, { throwIfNoEntry: false })?.isDirectory()) {
    throw new Error(`Tauri bundle directory does not exist: ${bundleDir}.`);
  }
  const bundles = readdirSync(bundleDir).filter((name) => config.pattern.test(name));
  if (bundles.length !== 1) {
    throw new Error(`Expected one updater bundle in ${bundleDir}; found ${bundles.length}.`);
  }

  const bundle = bundles[0];
  const signaturePath = path.join(bundleDir, `${bundle}.sig`);
  if (!statSync(signaturePath, { throwIfNoEntry: false })?.isFile()) {
    throw new Error(`Updater signature is missing: ${signaturePath}.`);
  }

  const destination = path.resolve("ci-artifacts");
  mkdirSync(destination, { recursive: true });
  copyFileSync(path.join(bundleDir, bundle), path.join(destination, bundle));
  copyFileSync(signaturePath, path.join(destination, `${bundle}.sig`));
  console.log(`Staged signed updater bundle ${bundle}.`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    stage();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

export { releaseTargets };
