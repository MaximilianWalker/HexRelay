import path from "node:path";
import { readText, replaceInFile, replaceText, writeText } from "./files.mjs";

export async function responseBuilderSuccessSchema(repoDir, mutationName) {
  if (mutationName !== "fail-response-builder-success-schema") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await replaceInFile(
    repoDir,
    "docs/contracts/runtime-rest.openapi.yaml",
    "                $ref: '#/components/schemas/TestingSessionCreateResponse'",
    "                $ref: '#/components/schemas/AuthVerifyResponse'",
    "fixture mutation target",
  );
}

export async function requestBodyMediaType(repoDir, mutationName) {
  if (mutationName !== "fail-request-body-media-type") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await replaceInFile(
    repoDir,
    "docs/contracts/runtime-rest.openapi.yaml",
    `        content:
          application/json:
            schema:
              $ref: '#/components/schemas/FriendRequestAcceptRequest'`,
    `        content:
          application/json:
            schema:
              $ref: '#/components/schemas/FriendRequestAcceptRequest'
          text/plain:
            schema:
              type: string`,
    "fixture mutation target",
  );
}

export async function apiErrorSchemaShape(repoDir, mutationName) {
  if (mutationName !== "fail-api-error-schema-shape") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await replaceInFile(
    repoDir,
    "docs/contracts/runtime-rest.openapi.yaml",
    `    ApiError:
      type: object
      required: [code, message]`,
    `    ApiError:
      type: object
      required: [code]`,
    "fixture mutation target",
  );
}

export async function serverChannelRequestSchema(repoDir, mutationName) {
  const modelsPath = path.join(repoDir, "services/api-rs/src/models.rs");
  const contractPath = path.join(repoDir, "docs/contracts/runtime-rest.openapi.yaml");
  const modelsText = await readText(modelsPath);
  const contractText = await readText(contractPath);

  if (mutationName === "fail-rest-schema-serde-default-required") {
    await writeText(
      modelsPath,
      replaceText(
        modelsText,
        "    pub mention_identity_ids: Option<Vec<String>>,",
        "    #[serde(default)]\n    pub mention_identity_ids: Vec<String>,",
        "fixture model mutation target",
      ),
    );
    await writeText(
      contractPath,
      replaceText(contractText, "      required: [content]", "      required: [content, mention_identity_ids]", "fixture contract required mutation target"),
    );
    return;
  }

  if (mutationName === "fail-rest-schema-array-item-pattern") {
    await writeText(
      contractPath,
      replaceText(contractText, "            pattern: '^[A-Za-z0-9_-]{3,64}$'", "", "fixture contract item-pattern mutation target"),
    );
    return;
  }

  throw new Error(`unknown fixture mutation: ${mutationName}`);
}
