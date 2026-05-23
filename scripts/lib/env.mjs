import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fromRoot, rootDir } from "./paths.mjs";

export function ensureCargoBinOnPath() {
  const cargoBin = path.join(os.homedir(), ".cargo", "bin");
  const delimiter = path.delimiter;
  const entries = (process.env.PATH ?? "").split(delimiter);
  if (!entries.includes(cargoBin)) {
    process.env.PATH = `${cargoBin}${delimiter}${process.env.PATH ?? ""}`;
  }
}

export function ensureFileFromExample(filePath, examplePath, label = "setup") {
  const resolvedPath = path.resolve(rootDir, filePath);
  const resolvedExample = path.resolve(rootDir, examplePath);
  if (fs.existsSync(resolvedPath)) {
    return;
  }

  console.log(`[${label}] ${filePath} missing; creating from ${examplePath}`);
  fs.copyFileSync(resolvedExample, resolvedPath);
}

export function readEnvFile(filePath) {
  const resolvedPath = path.resolve(rootDir, filePath);
  const values = {};
  const content = fs.readFileSync(resolvedPath, "utf8").replace(/^\uFEFF/, "");

  for (const rawLine of content.split(/\r?\n/)) {
    let line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }

    if (line.startsWith("export ")) {
      line = line.slice("export ".length).trim();
    }

    const separator = line.indexOf("=");
    if (separator === -1) {
      continue;
    }

    const key = line.slice(0, separator).trim();
    let value = line.slice(separator + 1).trim();
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) {
      throw new Error(`Invalid env key '${key}' in ${filePath}`);
    }

    if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
      value = value.slice(1, -1);
    }

    values[key] = value;
  }

  return values;
}

export function loadEnvFile(filePath, label = "env") {
  const values = readEnvFile(filePath);
  for (const [key, value] of Object.entries(values)) {
    process.env[key] = value;
  }
  console.log(`[${label}] Loaded env from ${filePath}`);
}

export function ensureAndLoadEnvFiles(files, label) {
  for (const file of files) {
    ensureFileFromExample(file.path, file.example, label);
    loadEnvFile(file.path, label);
  }
}

export function defaultRuntimeEnvFiles() {
  return [
    { path: "infra/.env", example: "infra/.env.example" },
    { path: "services/api-rs/.env", example: "services/api-rs/.env.example" },
  ];
}

export function fixturePath(...parts) {
  return fromRoot("fixtures", ...parts);
}
