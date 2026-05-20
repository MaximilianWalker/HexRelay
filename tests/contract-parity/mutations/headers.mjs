import path from "node:path";
import { appendToFile, readText, replaceInFile, replaceText, writeText } from "./files.mjs";

export async function responseHeaderSchemaType(repoDir, mutationName) {
  const contractPath = "docs/contracts/runtime-rest.openapi.yaml";
  const oldValue = `schema:
                type: string`;

  if (mutationName === "fail-response-header-schema-type") {
    await replaceInFile(
      repoDir,
      contractPath,
      oldValue,
      `schema:
                type: integer`,
      "fixture mutation target",
    );
    return;
  }

  if (mutationName === "pass-response-header-schema-ref") {
    await replaceInFile(
      repoDir,
      contractPath,
      oldValue,
      `schema:
                $ref: '#/components/schemas/CookieHeader'`,
      "fixture mutation target",
    );
    await appendToFile(
      repoDir,
      contractPath,
      `
    CookieHeader:
      type: string
`,
    );
    return;
  }

  throw new Error(`unknown fixture mutation: ${mutationName}`);
}

export async function unexpectedRequestHeader(repoDir, mutationName) {
  const contractPath = "docs/contracts/runtime-rest.openapi.yaml";
  const oldValue = `      parameters:
        - in: query
          name: cursor`;
  let newValue = "";
  let contractText = await readText(path.join(repoDir, contractPath));

  if (mutationName === "fail-unexpected-csrf-header") {
    newValue = `      parameters:
        - $ref: '#/components/parameters/CsrfTokenHeader'
        - in: query
          name: cursor`;
  } else if (mutationName === "fail-unexpected-internal-auth-header") {
    newValue = `      parameters:
        - in: header
          name: x-hexrelay-internal-token
          required: true
          schema:
            type: string
        - in: query
          name: cursor`;
  } else if (mutationName === "fail-unexpected-internal-auth-header-component") {
    newValue = `      parameters:
        - $ref: '#/components/parameters/InternalTokenHeader'
        - in: query
          name: cursor`;
    contractText = replaceText(
      contractText,
      `    CsrfTokenHeader:
      in: header
      name: x-csrf-token
      schema:
        type: string`,
      `    CsrfTokenHeader:
      in: header
      name: x-csrf-token
      schema:
        type: string
    InternalTokenHeader:
      in: header
      name: x-hexrelay-internal-token
      required: true
      schema:
        type: string`,
      "fixture component mutation target",
    );
  } else {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await writeText(path.join(repoDir, contractPath), replaceText(contractText, oldValue, newValue, "fixture mutation target"));
}

export async function delegatedCsrf(repoDir, mutationName) {
  if (mutationName !== "pass-delegated-csrf-header") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await replaceInFile(
    repoDir,
    "services/api-rs/src/transport/http/handlers/friends.rs",
    `    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    helper_accept()?;`,
    `    delegated_accept_guard(&auth, &headers)?;
    helper_accept()?;`,
    "fixture delegated csrf mutation target",
  );
  await appendToFile(
    repoDir,
    "services/api-rs/src/transport/http/handlers/friends.rs",
    `

fn delegated_accept_guard(auth: &AuthSession, headers: &HeaderMap) -> ApiResult<()> {
    enforce_csrf_for_cookie_auth(auth, headers)
}
`,
  );
}

export async function delegatedInternalHeader(repoDir, mutationName) {
  if (mutationName !== "pass-delegated-internal-auth-header") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await replaceInFile(
    repoDir,
    "services/api-rs/src/transport/http/handlers/friends.rs",
    `pub async fn forward_raw_body(
    State(_state): State<AppState>,
    _body: Bytes,
) -> ApiResult<Json<FriendRequestRecord>> {
    Ok(Json(FriendRequestRecord { id: "raw_1".to_string() }))
}`,
    `pub async fn forward_raw_body(
    State(_state): State<AppState>,
    headers: HeaderMap,
    _body: Bytes,
) -> ApiResult<Json<FriendRequestRecord>> {
    let _token = delegated_internal_header(&headers);
    Ok(Json(FriendRequestRecord { id: "raw_1".to_string() }))
}

fn delegated_internal_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-hexrelay-internal-token")
        .and_then(|value| value.to_str().ok())
}`,
    "fixture delegated internal handler target",
  );

  await replaceInFile(
    repoDir,
    "docs/contracts/runtime-rest.openapi.yaml",
    `  /internal/raw-forward:
    post:
      requestBody:`,
    `  /internal/raw-forward:
    post:
      parameters:
        - in: header
          name: x-hexrelay-internal-token
          required: true
          schema:
            type: string
      requestBody:`,
    "fixture delegated internal contract target",
  );
}
