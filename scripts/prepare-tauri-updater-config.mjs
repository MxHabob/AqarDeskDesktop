#!/usr/bin/env node
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export function normalizeUpdaterPublicKey(rawValue) {
  if (!rawValue?.trim()) {
    throw new Error("TAURI_UPDATER_PUBLIC_KEY is required for signed production releases.");
  }
  const value = rawValue.trim();
  let encoded;
  let publicKey;
  if (value.startsWith("untrusted comment:")) {
    publicKey = value.replace(/\r\n/g, "\n");
    encoded = Buffer.from(publicKey, "utf8").toString("base64");
  } else {
    encoded = value;
    if (!/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(encoded)) {
      throw new Error("TAURI_UPDATER_PUBLIC_KEY must contain the Tauri-generated .pub key.");
    }
    const decoded = Buffer.from(encoded, "base64");
    if (decoded.toString("base64") !== encoded) throw new Error("TAURI_UPDATER_PUBLIC_KEY is not canonical base64.");
    publicKey = decoded.toString("utf8").replace(/\r\n/g, "\n").trim();
  }
  const [comment, keyLine, ...extraLines] = publicKey.split("\n");
  const keyBytes = Buffer.from(keyLine ?? "", "base64");
  if (
    !/^untrusted comment:\s+\S/.test(comment ?? "") ||
    !/^[A-Za-z0-9+/]+={0,2}$/.test(keyLine ?? "") ||
    keyBytes.length !== 42 ||
    keyBytes.toString("base64") !== keyLine ||
    extraLines.length
  ) {
    throw new Error("TAURI_UPDATER_PUBLIC_KEY is not a valid minisign public key.");
  }
  return encoded;
}

function prepare() {
  const publicKey = normalizeUpdaterPublicKey(process.env.TAURI_UPDATER_PUBLIC_KEY);
  const configPath = path.resolve(
    process.env.TAURI_UPDATER_CONFIG_PATH ??
      "apps/desktop/src-tauri/target/tauri-updater-release.conf.json",
  );
  mkdirSync(path.dirname(configPath), { recursive: true });
  writeFileSync(
    configPath,
    `${JSON.stringify({
      bundle: { createUpdaterArtifacts: true },
      plugins: { updater: { pubkey: publicKey } },
    }, null, 2)}\n`,
    { mode: 0o600 },
  );
  console.log("Prepared temporary signed-updater configuration.");
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    prepare();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
