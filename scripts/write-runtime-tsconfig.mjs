import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const instance = process.argv[2] ?? process.env.HEXRELAY_RUNTIME_INSTANCE;

if (!instance || !/^[a-zA-Z0-9_-]+$/.test(instance)) {
  console.error("[runtime-tsconfig] ERROR: runtime instance must match /^[a-zA-Z0-9_-]+$/");
  process.exit(1);
}

const webDir = path.join(process.cwd(), "apps", "web");
const outputDir = path.join(webDir, ".runtime-tsconfig");
const outputPath = path.join(outputDir, `${instance}.json`);

const config = {
  extends: "../tsconfig.json",
  include: [
    "../next-env.d.ts",
    "../**/*.ts",
    "../**/*.tsx",
    `../.next-${instance}/types/**/*.ts`,
    `../.next-${instance}/dev/types/**/*.ts`,
    "../**/*.mts",
  ],
  exclude: ["../node_modules"],
};

fs.mkdirSync(outputDir, { recursive: true });
fs.writeFileSync(outputPath, `${JSON.stringify(config, null, 2)}\n`);
console.log(`[runtime-tsconfig] Wrote ${path.relative(process.cwd(), outputPath)}`);
