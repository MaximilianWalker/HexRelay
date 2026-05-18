import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import ts from "typescript";
import { describe, expect, it } from "vitest";

const webRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoots = ["app", "components"].map((sourceRoot) => path.join(webRoot, sourceRoot));

type Finding = {
  file: string;
  line: number;
  tagName: string;
};

function listTsxFiles(root: string): string[] {
  const entries = fs.readdirSync(root, { withFileTypes: true });
  const files: string[] = [];

  for (const entry of entries) {
    const fullPath = path.join(root, entry.name);

    if (entry.isDirectory()) {
      files.push(...listTsxFiles(fullPath));
      continue;
    }

    if (entry.isFile() && entry.name.endsWith(".tsx")) {
      files.push(fullPath);
    }
  }

  return files;
}

function attributeByName(
  attributes: ts.JsxAttributes,
  name: string,
): ts.JsxAttribute | undefined {
  return attributes.properties.find(
    (attribute): attribute is ts.JsxAttribute =>
      ts.isJsxAttribute(attribute) && attribute.name.text === name,
  );
}

function attributeHasValue(attribute: ts.JsxAttribute | undefined): boolean {
  if (!attribute?.initializer) {
    return false;
  }

  if (ts.isStringLiteral(attribute.initializer)) {
    return attribute.initializer.text.trim().length > 0;
  }

  if (!ts.isJsxExpression(attribute.initializer)) {
    return false;
  }

  const expression = attribute.initializer.expression;
  if (!expression) {
    return false;
  }

  if (ts.isStringLiteralLike(expression)) {
    return expression.text.trim().length > 0;
  }

  return true;
}

function staticAttributeValue(
  attributes: ts.JsxAttributes,
  name: string,
): string | null {
  const attribute = attributeByName(attributes, name);
  if (!attribute?.initializer) {
    return null;
  }

  if (ts.isStringLiteral(attribute.initializer)) {
    return attribute.initializer.text;
  }

  if (ts.isJsxExpression(attribute.initializer)) {
    const expression = attribute.initializer.expression;
    return expression && ts.isStringLiteralLike(expression) ? expression.text : null;
  }

  return null;
}

function tagNameText(tagName: ts.JsxTagNameExpression): string | null {
  return ts.isIdentifier(tagName) ? tagName.text : null;
}

function collectLabelTargets(sourceFile: ts.SourceFile): Set<string> {
  const labelTargets = new Set<string>();

  function visit(node: ts.Node): void {
    if (ts.isJsxOpeningElement(node) && tagNameText(node.tagName) === "label") {
      const target = staticAttributeValue(node.attributes, "htmlFor");
      if (target) {
        labelTargets.add(target);
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return labelTargets;
}

function isHiddenInput(node: ts.JsxOpeningLikeElement, tagName: string): boolean {
  return tagName === "input" && staticAttributeValue(node.attributes, "type") === "hidden";
}

function hasAccessibleName(
  node: ts.JsxOpeningLikeElement,
  labelTargets: Set<string>,
): boolean {
  if (
    attributeHasValue(attributeByName(node.attributes, "aria-label")) ||
    attributeHasValue(attributeByName(node.attributes, "aria-labelledby"))
  ) {
    return true;
  }

  const id = staticAttributeValue(node.attributes, "id");
  return Boolean(id && labelTargets.has(id));
}

function collectFindings(filePath: string): Finding[] {
  const source = fs.readFileSync(filePath, "utf8");
  const sourceFile = ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
  const labelTargets = collectLabelTargets(sourceFile);
  const findings: Finding[] = [];

  function visit(node: ts.Node): void {
    if (ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) {
      const tagName = tagNameText(node.tagName);

      if (
        tagName &&
        ["input", "textarea"].includes(tagName) &&
        !isHiddenInput(node, tagName) &&
        !hasAccessibleName(node, labelTargets)
      ) {
        const line = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile)).line + 1;
        findings.push({
          file: path.relative(webRoot, filePath),
          line,
          tagName,
        });
      }
    }

    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return findings;
}

describe("text-entry control accessibility", () => {
  it("keeps inputs and textareas reachable by durable accessible names", () => {
    const findings = sourceRoots.flatMap((sourceRoot) =>
      listTsxFiles(sourceRoot).flatMap(collectFindings),
    );

    expect(findings).toEqual([]);
  });
});
