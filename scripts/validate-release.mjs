#!/usr/bin/env node
import { appendFileSync, readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import path from "node:path";

function read(relativePath) {
  return readFileSync(path.resolve(relativePath), "utf8");
}

function requireVersion(condition, message) {
  if (!condition) throw new Error(message);
}

const requestedTag = (process.env.TAG_INPUT ?? "").trim();
const tag = /^v/.test(requestedTag) ? requestedTag : `v${requestedTag}`;
requireVersion(/^v\d+\.\d+\.\d+$/.test(tag), "Release tag must use vMAJOR.MINOR.PATCH.");
const version = tag.slice(1);

const desktopPackage = JSON.parse(read("apps/desktop/package.json"));
const tauriConfig = JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
const cargo = read("apps/desktop/src-tauri/Cargo.toml");
const cargoLock = read("Cargo.lock");
const cargoVersion = /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1];
const lockVersion = /^\[\[package\]\]\r?\nname = "aqardesk-desktop"\r?\nversion = "([^"]+)"/m.exec(cargoLock)?.[1];
for (const [label, current] of Object.entries({
  "desktop package": desktopPackage.version,
  Tauri: tauriConfig.version,
  Cargo: cargoVersion,
  "Cargo.lock": lockVersion,
})) {
  requireVersion(current === version, `Tag ${tag} does not match ${label} version ${current ?? "(missing)"}.`);
}

const commitSha = execFileSync("git", ["rev-list", "-n", "1", tag], { encoding: "utf8" }).trim();
execFileSync("git", ["merge-base", "--is-ancestor", commitSha, "origin/main"], { stdio: "ignore" });

const notesPath = `releases/${tag}.md`;
const notes = read(notesPath);
requireVersion(
  notes.startsWith(`# AqarDesk Desktop ${tag}`),
  `${notesPath} must begin with "# AqarDesk Desktop ${tag}".`,
);
requireVersion(
  !/<!--|Replace this prompt|Add user-facing changes|Write None or list/i.test(notes),
  `${notesPath} still contains release-note placeholders.`,
);
requireVersion(/## Summary\s*\n\s*\S/.test(notes), `${notesPath} must contain a user-facing summary.`);
requireVersion(process.env.GITHUB_OUTPUT, "GITHUB_OUTPUT is unavailable.");
appendFileSync(process.env.GITHUB_OUTPUT, `tag=${tag}\nversion=${version}\ncommit_sha=${commitSha}\n`);
console.log(`Validated AqarDesk release ${tag} at ${commitSha}.`);
