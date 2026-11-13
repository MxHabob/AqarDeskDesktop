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
    suffix: "-aarch64",
  },
  "x86_64-apple-darwin": {
    bundle: "macos",
    pattern: /\.app\.tar\.gz$/,
    suffix: "-x86_64",
  },
  "x86_64-unknown-linux-gnu": {
    bundle: "appimage",
    pattern: /\.AppImage\.tar\.gz$/,
  },
};

function stage({
  root = process.cwd(),
  target = process.env.TAURI_TARGET,
  bundle: requestedBundle = process.env.TAURI_BUNDLE,
} = {}) {
  const config = releaseTargets[target];
  if (!config) throw new Error(`Unsupported desktop release target: ${target ?? "(missing)"}.`);
  if (requestedBundle !== config.bundle) {
    throw new Error(`Expected ${config.bundle} bundle directory for ${target}.`);
  }

  const bundleDir = path.resolve(
    root,
    `target/${target}/release/bundle/${config.bundle}`,
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

  const stagedBundle = config.suffix
    ? bundle.replace(/\.app\.tar\.gz$/, `${config.suffix}.app.tar.gz`)
    : bundle;
  const destination = path.resolve(root, "ci-artifacts");
  mkdirSync(destination, { recursive: true });
  copyFileSync(path.join(bundleDir, bundle), path.join(destination, stagedBundle));
  copyFileSync(signaturePath, path.join(destination, `${stagedBundle}.sig`));
  console.log(`Staged signed updater bundle ${stagedBundle}.`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    stage();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

export { releaseTargets, stage };
