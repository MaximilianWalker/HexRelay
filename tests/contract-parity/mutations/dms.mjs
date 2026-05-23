import path from "node:path";
import { appendToFile, readText, replaceInFile, replaceText, writeText } from "./files.mjs";

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
