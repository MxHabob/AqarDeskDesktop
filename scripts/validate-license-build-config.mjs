#!/usr/bin/env node

const required = [
  "MULVERSE_LICENSE_API_BASE_URL",
  "MULVERSE_LICENSE_PRODUCT_CODE",
  "MULVERSE_LICENSE_PRODUCT_ID",
  "MULVERSE_LICENSE_KEY_PREFIX",
  "MULVERSE_LICENSE_PUBLIC_KEY_ID",
  "MULVERSE_LICENSE_PUBLIC_KEY_B64",
];

export function validateLicenseBuildConfig(env = process.env) {
  const missing = required.filter((name) => !env[name]?.trim());
  if (missing.length) {
    throw new Error(`Missing Mulverse license build variables: ${missing.join(", ")}.`);
  }

  let apiUrl;
  try {
    apiUrl = new URL(env.MULVERSE_LICENSE_API_BASE_URL);
  } catch {
    throw new Error("MULVERSE_LICENSE_API_BASE_URL must be a valid HTTPS URL.");
  }
  if (apiUrl.protocol !== "https:") {
    throw new Error("MULVERSE_LICENSE_API_BASE_URL must be a valid HTTPS URL.");
  }

  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(env.MULVERSE_LICENSE_PRODUCT_ID)) {
    throw new Error("MULVERSE_LICENSE_PRODUCT_ID must be a valid UUID.");
  }

  const key = env.MULVERSE_LICENSE_PUBLIC_KEY_B64;
  if (
    !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(key)
    || Buffer.from(key, "base64").length !== 32
    || Buffer.from(key, "base64").toString("base64") !== key
  ) {
    throw new Error("MULVERSE_LICENSE_PUBLIC_KEY_B64 must be canonical base64 for a 32-byte Ed25519 public key.");
  }
}

if (process.argv[1] && process.argv[1].endsWith("validate-license-build-config.mjs")) {
  try {
    validateLicenseBuildConfig();
    console.log("Mulverse license build configuration is valid.");
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
