#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const files = {
  package: "apps/desktop/package.json",
  cargo: "apps/desktop/src-tauri/Cargo.toml",
  cargoLock: "Cargo.lock",
  tauri: "apps/desktop/src-tauri/tauri.conf.json",
  devTauri: "apps/desktop/src-tauri/tauri.dev.conf.json",
};

function read(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), "utf8");
}

function parseVersion(value) {
  const match = /^v?(\d+)\.(\d+)\.(\d+)$/.exec(value ?? "");
  if (!match) {
    throw new Error(`Expected a stable SemVer version (x.y.z), received "${value}".`);
  }
  return match.slice(1).map(Number);
}

function compareVersions(left, right) {
  const a = parseVersion(left);
  const b = parseVersion(right);
  for (let index = 0; index < a.length; index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return 0;
}

function bumpVersion(currentVersion, level) {
  const [major, minor, patch] = parseVersion(currentVersion);
  if (level === "major") return `${major + 1}.0.0`;
  if (level === "minor") return `${major}.${minor + 1}.0`;
  if (level === "patch") return `${major}.${minor}.${patch + 1}`;
  throw new Error(`Unknown bump type "${level}". Use patch, minor, major, or an explicit x.y.z version.`);
}

function updateTomlPackageVersion(source, version) {
  const pattern = /(^\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m;
  if (!pattern.test(source)) throw new Error("Could not find [package].version in desktop Cargo.toml.");
  return source.replace(pattern, (_match, prefix, _current, suffix) => `${prefix}${version}${suffix}`);
}

function updateJsonVersion(source, version) {
  const pattern = /^([ \t]*"version"\s*:\s*")([^"]+)(")/m;
  if (!pattern.test(source)) throw new Error("Could not find root version in desktop JSON configuration.");
  return source.replace(pattern, (_match, prefix, _current, suffix) => `${prefix}${version}${suffix}`);
}

function updateCargoLockVersion(source, version) {
  const pattern = /(^\[\[package\]\]\r?\nname = "aqardesk-desktop"\r?\nversion = ")([^"]+)(")/m;
  if (!pattern.test(source)) throw new Error("Could not find aqardesk-desktop in Cargo.lock.");
  return source.replace(pattern, (_match, prefix, _current, suffix) => `${prefix}${version}${suffix}`);
}

function latestTag() {
  return execFileSync("git", ["tag", "--list", "v[0-9]*", "--sort=-version:refname"], {
    cwd: root,
    encoding: "utf8",
  })
    .split(/\r?\n/)
    .map((tag) => tag.trim())
    .find((tag) => /^v\d+\.\d+\.\d+$/.test(tag)) ?? null;
}

function getCommits(tag) {
  const args = ["log", "--no-merges", "--format=%s%x1f%b%x1e"];
  if (tag) args.splice(1, 0, `${tag}..HEAD`);
  else args.splice(1, 0, "-n", "100");
  return execFileSync("git", args, { cwd: root, encoding: "utf8" })
    .split("\x1e")
    .map((record) => record.trim())
    .filter(Boolean)
    .map((record) => {
      const [subject = "", ...body] = record.split("\x1f");
      return { subject: subject.trim(), body: body.join("\x1f").trim() };
    });
}

function classifyCommit({ subject, body }) {
  const match = /^(feat|fix|perf|refactor|docs|build|ci|test|chore)(?:\(([^)]+)\))?(!)?:\s*(.+)$/i.exec(subject);
  const category = match?.[3] || /BREAKING CHANGE:/i.test(body)
    ? "breaking"
    : match?.[1]?.toLowerCase() ?? "other";
  const scope = match?.[2] ? ` (${match[2]})` : "";
  return { category, entry: `- ${match?.[4] ?? subject}${scope}` };
}

function renderReleaseNotes(version, commits) {
  const sections = [
    ["breaking", "Breaking Changes"],
    ["feat", "Added"],
    ["fix", "Fixed"],
    ["perf", "Performance"],
    ["refactor", "Changed"],
    ["docs", "Documentation"],
    ["build", "Build"],
    ["ci", "CI"],
    ["test", "Tests"],
    ["chore", "Maintenance"],
    ["other", "Other"],
  ];
  const grouped = new Map(sections.map(([key]) => [key, []]));
  for (const commit of commits) {
    const classified = classifyCommit(commit);
    grouped.get(classified.category).push(classified.entry);
  }

  const lines = [
    `# AqarDesk Desktop v${version}`,
    "",
    "## Summary",
    "<!-- Replace this prompt with a concise, user-facing summary before release. -->",
    "",
  ];
  for (const [key, title] of sections) {
    const entries = grouped.get(key);
    if (entries.length) lines.push(`## ${title}`, "", ...entries, "");
  }
  if (!commits.length) {
    lines.push("## Changes", "", "- Add user-facing changes for this release.", "");
  }
  lines.push("## Known Issues", "", "<!-- Write None or list known issues. -->", "");
  return lines.join("\n");
}

function prepare() {
  const args = process.argv.slice(2);
  const dryRun = args.includes("--dry-run");
  if (args.some((arg) => arg.startsWith("--") && arg !== "--dry-run")) {
    throw new Error("Supported option: --dry-run. Provide patch, minor, major, or an explicit x.y.z version.");
  }
  const versionArgs = args.filter((arg) => arg !== "--dry-run");
  if (versionArgs.length > 1) throw new Error("Provide one version bump type or one explicit SemVer version.");

  const packageContents = read(files.package);
  const packageJson = JSON.parse(packageContents);
  const cargo = read(files.cargo);
  const cargoLock = read(files.cargoLock);
  const tauriContents = read(files.tauri);
  const tauri = JSON.parse(tauriContents);
  const devTauriContents = read(files.devTauri);
  const devTauri = JSON.parse(devTauriContents);
  const currentVersions = {
    package: packageJson.version,
    cargo: /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1],
    cargoLock: /^\[\[package\]\]\r?\nname = "aqardesk-desktop"\r?\nversion = "([^"]+)"/m.exec(cargoLock)?.[1],
    tauri: tauri.version,
    devTauri: devTauri.version,
  };
  if (Object.values(currentVersions).some((version) => !version || version !== currentVersions.package)) {
    throw new Error(`Desktop version files are inconsistent: ${JSON.stringify(currentVersions)}.`);
  }

  const taggedVersion = latestTag();
  const baseline = taggedVersion && compareVersions(taggedVersion, currentVersions.package) > 0
    ? taggedVersion.slice(1)
    : currentVersions.package;
  const requested = versionArgs[0] ?? "patch";
  const nextVersion = ["patch", "minor", "major"].includes(requested)
    ? bumpVersion(baseline, requested)
    : requested.replace(/^v/, "");
  parseVersion(nextVersion);
  const isInitialRelease = !taggedVersion
    && !["patch", "minor", "major"].includes(requested)
    && compareVersions(nextVersion, currentVersions.package) === 0;
  if (compareVersions(nextVersion, baseline) <= 0 && !isInitialRelease) {
    throw new Error(`New version ${nextVersion} must be greater than current/latest version ${baseline}.`);
  }

  const tag = `v${nextVersion}`;
  if (execFileSync("git", ["tag", "--list", tag], { cwd: root, encoding: "utf8" }).trim()) {
    throw new Error(`Release tag ${tag} already exists.`);
  }
  const notesPath = path.join(root, "releases", `${tag}.md`);
  if (fs.existsSync(notesPath)) throw new Error(`Release notes already exist: releases/${tag}.md.`);

  const updated = {
    [files.package]: updateJsonVersion(packageContents, nextVersion),
    [files.cargo]: updateTomlPackageVersion(cargo, nextVersion),
    [files.cargoLock]: updateCargoLockVersion(cargoLock, nextVersion),
    [files.tauri]: updateJsonVersion(tauriContents, nextVersion),
    [files.devTauri]: updateJsonVersion(devTauriContents, nextVersion),
  };

  if (dryRun) {
    console.log(`Planned AqarDesk release version: ${currentVersions.package} -> ${nextVersion}.`);
    for (const file of Object.keys(updated)) console.log(`- ${file}`);
    console.log(`- releases/${tag}.md`);
    return;
  }

  const status = execFileSync("git", ["status", "--porcelain"], { cwd: root, encoding: "utf8" });
  if (status.trim()) throw new Error("Release preparation requires a clean working tree. Commit or stash changes first.");

  const previousTag = taggedVersion;
  const notes = renderReleaseNotes(nextVersion, getCommits(previousTag));
  for (const [relativePath, contents] of Object.entries(updated)) {
    fs.writeFileSync(path.join(root, relativePath), contents);
  }
  fs.mkdirSync(path.dirname(notesPath), { recursive: true });
  fs.writeFileSync(notesPath, notes);
  console.log(`Prepared AqarDesk Desktop v${nextVersion}. Review releases/${tag}.md, commit the changes, and push tag ${tag} to publish.`);
}

export {
  bumpVersion,
  classifyCommit,
  compareVersions,
  parseVersion,
  renderReleaseNotes,
  updateCargoLockVersion,
  updateTomlPackageVersion,
};

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    prepare();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
