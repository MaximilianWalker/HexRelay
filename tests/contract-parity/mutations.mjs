import fs from "node:fs/promises";
import path from "node:path";

async function readText(filePath) {
  return (await fs.readFile(filePath, "utf8")).replace(/\r\n/g, "\n");
}

async function writeText(filePath, text) {
  await fs.writeFile(filePath, text, "utf8");
}

function replaceText(text, oldValue, newValue, targetLabel) {
  if (!text.includes(oldValue)) {
    throw new Error(`${targetLabel} not found`);
  }
  return text.replace(oldValue, newValue);
}

async function replaceInFile(repoDir, relativePath, oldValue, newValue, targetLabel) {
  const filePath = path.join(repoDir, relativePath);
  const text = await readText(filePath);
  await writeText(filePath, replaceText(text, oldValue, newValue, targetLabel));
}

async function appendToFile(repoDir, relativePath, value) {
  const filePath = path.join(repoDir, relativePath);
  await writeText(filePath, `${await readText(filePath)}${value}`);
}

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

export async function pathParameterFormat(repoDir, mutationName) {
  await replaceInFile(
    repoDir,
    "services/api-rs/src/transport/http/handlers/friends.rs",
    "Path(_request_id): Path<String>,",
    "Path(_request_id): Path<uuid::Uuid>,",
    "fixture handler mutation target",
  );

  if (mutationName === "pass-path-parameter-format") {
    await replaceInFile(
      repoDir,
      "docs/contracts/runtime-rest.openapi.yaml",
      `        - in: path
          name: request_id
          required: true
          schema:
            type: string`,
      `        - in: path
          name: request_id
          required: true
          schema:
            type: string
            format: uuid`,
      "fixture contract mutation target",
    );
    return;
  }

  if (mutationName !== "fail-path-parameter-format") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }
}

export async function queryParameterPattern(repoDir, mutationName) {
  if (mutationName !== "fail-query-parameter-pattern") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await replaceInFile(
    repoDir,
    "services/api-rs/src/app/router.rs",
    '        .route("/dm/threads", get(list_dm_threads));',
    `        .route("/dm/threads", get(list_dm_threads))
        .route("/friends/requests", get(list_friend_requests));`,
    "fixture router mutation target",
  );

  await appendToFile(
    repoDir,
    "services/api-rs/src/models.rs",
    `

pub struct FriendRequestListQuery {
    pub identity_id: String,
    pub direction: Option<String>,
}

pub struct FriendRequestPage {
    pub items: Vec<String>,
}
`,
  );

  const handlerPath = path.join(repoDir, "services/api-rs/src/transport/http/handlers/friends.rs");
  let handlerText = await readText(handlerPath);
  handlerText = replaceText(
    handlerText,
    "models::{DmThreadListQuery, DmThreadPage, FriendRequestAcceptRequest, FriendRequestRecord},",
    "models::{DmThreadListQuery, DmThreadPage, FriendRequestAcceptRequest, FriendRequestListQuery, FriendRequestPage, FriendRequestRecord},",
    "fixture handler import mutation target",
  );
  handlerText += `

pub async fn list_friend_requests(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Query(query): Query<FriendRequestListQuery>,
) -> ApiResult<Json<FriendRequestPage>> {
    if false {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    let _identity_id = query.identity_id;
    let _direction = query.direction;
    Ok(Json(FriendRequestPage { items: vec![] }))
}
`;
  await writeText(handlerPath, handlerText);

  const contractPath = path.join(repoDir, "docs/contracts/runtime-rest.openapi.yaml");
  let contractText = await readText(contractPath);
  const friendRoute = `  /friends/requests:
    get:
      security:
        - CookieAuth: []
        - BearerAuth: []
      parameters:
        - in: query
          name: identity_id
          required: true
          schema:
            type: string
        - in: query
          name: direction
          schema:
            type: string
            enum: [inbound, outbound]
      responses:
        '200':
          description: Friend request page
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/FriendRequestPage'
        '400':
          description: Invalid friend request query
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                identityInvalid:
                  value:
                    code: identity_invalid
                    message: identity_id must be 3-64 chars using letters, numbers, _ or -
        '401':
          description: Unauthorized
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                identityInvalid:
                  value:
                    code: identity_invalid
                    message: identity_id must match authenticated session
        '500':
          $ref: '#/components/responses/InternalServerError'
`;
  contractText = replaceText(contractText, "  /dm/threads:\n", `${friendRoute}  /dm/threads:\n`, "fixture contract mutation target");
  contractText = replaceText(
    contractText,
    `    DmThreadPage:
      type: object
`,
    `    DmThreadPage:
      type: object
    FriendRequestPage:
      type: object
`,
    "fixture contract schema mutation target",
  );
  await writeText(contractPath, contractText);
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

export async function dmPolicySchema(repoDir, mutationName) {
  await replaceInFile(
    repoDir,
    "services/api-rs/src/app/router.rs",
    'Router::new().route("/dm/threads", get(list_dm_threads));',
    `Router::new()
        .route("/dm/threads", get(list_dm_threads))
        .route("/dm/privacy-policy", post(update_dm_policy));`,
    "fixture router mutation target",
  );

  await appendToFile(
    repoDir,
    "services/api-rs/src/models.rs",
    `

pub struct DmPolicy {
    pub inbound_policy: String,
    pub offline_delivery_mode: String,
}

pub struct DmPolicyUpdate {
    pub inbound_policy: String,
}
`,
  );

  const handlerPath = path.join(repoDir, "services/api-rs/src/transport/http/handlers/dm.rs");
  let handlerText = await readText(handlerPath);
  handlerText = replaceText(
    handlerText,
    "models::{DmThreadListQuery, DmThreadPage},",
    "models::{DmPolicy, DmPolicyUpdate, DmThreadListQuery, DmThreadPage},",
    "fixture handler import mutation target",
  );
  handlerText += `

pub async fn update_dm_policy(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Json(payload): Json<DmPolicyUpdate>,
) -> ApiResult<Json<DmPolicy>> {
    if !matches!(
        payload.inbound_policy.as_str(),
        "friends_only" | "same_server" | "anyone"
    ) {
        return Err(bad_request(
            "dm_policy_invalid",
            "inbound_policy must be one of: friends_only, same_server, anyone",
        ));
    }
    Ok(Json(DmPolicy {
        inbound_policy: payload.inbound_policy,
        offline_delivery_mode: "encrypted_envelope_catchup".to_string(),
    }))
}
`;
  await writeText(handlerPath, handlerText);

  const contractPath = path.join(repoDir, "docs/contracts/runtime-rest.openapi.yaml");
  let contractText = await readText(contractPath);
  const routeInsert = `  /dm/privacy-policy:
    post:
      security:
        - CookieAuth: []
        - BearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/DmPolicyUpdate'
      responses:
        '200':
          description: DM policy updated
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DmPolicy'
        '400':
          description: Invalid DM policy
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                dmPolicyInvalid:
                  value:
                    code: dm_policy_invalid
                    message: "inbound_policy must be one of: friends_only, same_server, anyone"
        '401':
          $ref: '#/components/responses/Unauthorized'
        '500':
          $ref: '#/components/responses/InternalServerError'
`;
  contractText = replaceText(contractText, "components:\n", `${routeInsert}components:\n`, "fixture contract components target");
  const schemaInsert = `    DmPolicy:
      type: object
      required: [inbound_policy, offline_delivery_mode]
      properties:
        inbound_policy:
          type: string
          enum: [friends_only, same_server, anyone]
        offline_delivery_mode:
          type: string
          enum: [encrypted_envelope_catchup]
    DmPolicyUpdate:
      type: object
      required: [inbound_policy]
      properties:
        inbound_policy:
          type: string
          enum: [friends_only, same_server, anyone]
`;
  contractText = replaceText(contractText, "    ApiError:\n", `${schemaInsert}    ApiError:\n`, "fixture contract schema target");
  contractText = replaceText(
    contractText,
    "            - cursor_invalid",
    "            - cursor_invalid\n            - dm_policy_invalid",
    "fixture contract error-code target",
  );

  if (mutationName === "fail-dm-policy-update-inbound-enum-domain") {
    const oldValue = "          enum: [friends_only, same_server, anyone]";
    const firstIndex = contractText.indexOf(oldValue);
    const secondIndex = contractText.indexOf(oldValue, firstIndex + oldValue.length);
    if (firstIndex === -1 || secondIndex === -1) {
      throw new Error("fixture DmPolicyUpdate enum mutation target not found");
    }
    contractText = `${contractText.slice(0, secondIndex)}          enum: [friends_only, same_server]${contractText.slice(
      secondIndex + oldValue.length,
    )}`;
  } else if (mutationName === "fail-dm-policy-response-offline-mode-enum-domain") {
    contractText = replaceText(
      contractText,
      `        offline_delivery_mode:
          type: string
          enum: [encrypted_envelope_catchup]`,
      `        offline_delivery_mode:
          type: string`,
      "fixture DmPolicy offline mode enum mutation target",
    );
  } else if (mutationName !== "pass-dm-policy-enum-domain") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await writeText(contractPath, contractText);
}

export async function dmMarkReadSchema(repoDir, mutationName) {
  await replaceInFile(
    repoDir,
    "services/api-rs/src/app/router.rs",
    'Router::new().route("/dm/threads", get(list_dm_threads));',
    `Router::new()
        .route("/dm/threads", get(list_dm_threads))
        .route("/dm/threads/:thread_id/read", post(mark_dm_thread_read));`,
    "fixture router mutation target",
  );

  await appendToFile(
    repoDir,
    "services/api-rs/src/models.rs",
    `

pub struct DmThreadMarkReadRequest {
    pub last_read_seq: u64,
}

pub struct DmThreadMarkReadResponse {
    pub thread_id: String,
    pub last_read_seq: u64,
    pub unread: u32,
}
`,
  );

  const handlerPath = path.join(repoDir, "services/api-rs/src/transport/http/handlers/dm.rs");
  let handlerText = await readText(handlerPath);
  handlerText = replaceText(
    handlerText,
    "use axum::{extract::{Query, State}, Json};",
    "use axum::{extract::{Path, Query, State}, http::StatusCode, Json};",
    "fixture handler import mutation target",
  );
  handlerText = replaceText(
    handlerText,
    "models::{DmThreadListQuery, DmThreadPage},",
    "models::{DmThreadListQuery, DmThreadMarkReadRequest, DmThreadMarkReadResponse, DmThreadPage},",
    "fixture handler model import mutation target",
  );
  handlerText += `

pub async fn mark_dm_thread_read(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Path(thread_id): Path<String>,
    Json(payload): Json<DmThreadMarkReadRequest>,
) -> ApiResult<Json<DmThreadMarkReadResponse>> {
    if payload.last_read_seq == u64::MAX {
        return Err(bad_request(
            "last_read_seq_invalid",
            "last_read_seq is outside the known thread sequence range",
        ));
    }
    if false {
        return Err((
            StatusCode::NOT_FOUND,
            Json(crate::shared::errors::ApiError {
                code: "thread_not_found",
                message: "thread was not found for this identity",
            }),
        ));
    }
    Ok(Json(DmThreadMarkReadResponse {
        thread_id,
        last_read_seq: payload.last_read_seq,
        unread: 0,
    }))
}
`;
  await writeText(handlerPath, handlerText);

  const contractPath = path.join(repoDir, "docs/contracts/runtime-rest.openapi.yaml");
  let contractText = await readText(contractPath);
  const routeInsert = `  /dm/threads/{thread_id}/read:
    post:
      security:
        - CookieAuth: []
        - BearerAuth: []
      parameters:
        - in: path
          name: thread_id
          required: true
          schema:
            type: string
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/DmThreadMarkReadRequest'
      responses:
        '200':
          description: Updated read position
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DmThreadMarkReadResponse'
        '400':
          description: Invalid read position
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                lastReadSeqInvalid:
                  value:
                    code: last_read_seq_invalid
                    message: last_read_seq is outside the known thread sequence range
        '401':
          $ref: '#/components/responses/Unauthorized'
        '404':
          description: Thread not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ApiError'
              examples:
                threadNotFound:
                  value:
                    code: thread_not_found
                    message: thread was not found for this identity
        '500':
          $ref: '#/components/responses/InternalServerError'
`;
  contractText = replaceText(contractText, "components:\n", `${routeInsert}components:\n`, "fixture contract components target");
  const schemaInsert = `    DmThreadMarkReadRequest:
      type: object
      required: [last_read_seq]
      properties:
        last_read_seq:
          type: integer
          minimum: 0
    DmThreadMarkReadResponse:
      type: object
      required: [thread_id, last_read_seq, unread]
      properties:
        thread_id:
          type: string
        last_read_seq:
          type: integer
          minimum: 0
        unread:
          type: integer
          minimum: 0
`;
  contractText = replaceText(contractText, "    ApiError:\n", `${schemaInsert}    ApiError:\n`, "fixture contract schema target");
  contractText = replaceText(
    contractText,
    "            - storage_unavailable",
    "            - storage_unavailable\n            - last_read_seq_invalid\n            - thread_not_found",
    "fixture contract error-code target",
  );

  if (mutationName === "fail-dm-mark-read-scalar-bounds") {
    contractText = replaceText(
      contractText,
      `        last_read_seq:
          type: integer
          minimum: 0`,
      `        last_read_seq:
          type: integer`,
      "fixture contract scalar-bound mutation target",
    );
  } else if (mutationName === "fail-dm-mark-read-response-last-read-scalar-bound") {
    const oldValue = `        last_read_seq:
          type: integer
          minimum: 0`;
    const firstIndex = contractText.indexOf(oldValue);
    const secondIndex = contractText.indexOf(oldValue, firstIndex + oldValue.length);
    if (firstIndex === -1 || secondIndex === -1) {
      throw new Error("fixture response last_read_seq scalar-bound mutation target not found");
    }
    contractText = `${contractText.slice(0, secondIndex)}        last_read_seq:
          type: integer${contractText.slice(secondIndex + oldValue.length)}`;
  } else if (mutationName === "fail-dm-mark-read-response-unread-scalar-bound") {
    contractText = replaceText(
      contractText,
      `        unread:
          type: integer
          minimum: 0`,
      `        unread:
          type: integer`,
      "fixture response unread scalar-bound mutation target",
    );
  } else if (mutationName !== "pass-dm-mark-read-scalar-bounds") {
    throw new Error(`unknown fixture mutation: ${mutationName}`);
  }

  await writeText(contractPath, contractText);
}
