import fs from "node:fs/promises";
import path from "node:path";

export async function readText(filePath) {
  return (await fs.readFile(filePath, "utf8")).replace(/\r\n/g, "\n");
}

export async function writeText(filePath, text) {
  await fs.writeFile(filePath, text, "utf8");
}

export function replaceText(text, oldValue, newValue, targetLabel) {
  if (!text.includes(oldValue)) {
    throw new Error(`${targetLabel} not found`);
  }
  return text.replace(oldValue, newValue);
}

export async function replaceInFile(repoDir, relativePath, oldValue, newValue, targetLabel) {
  const filePath = path.join(repoDir, relativePath);
  const text = await readText(filePath);
  await writeText(filePath, replaceText(text, oldValue, newValue, targetLabel));
}

export async function appendToFile(repoDir, relativePath, value) {
  const filePath = path.join(repoDir, relativePath);
  await writeText(filePath, `${await readText(filePath)}${value}`);
}
