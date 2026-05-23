import fs from "node:fs";
import path from "node:path";

export function stripBom(value) {
  return value.replace(/^\uFEFF/, "");
}

export function readJsonIfExists(filePath) {
  if (!fs.existsSync(filePath)) {
    return null;
  }
  return JSON.parse(stripBom(fs.readFileSync(filePath, "utf8")));
}

export function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}
