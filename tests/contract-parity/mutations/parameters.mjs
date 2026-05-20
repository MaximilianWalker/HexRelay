import path from "node:path";
import { appendToFile, readText, replaceInFile, replaceText, writeText } from "./files.mjs";

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
