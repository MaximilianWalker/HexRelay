import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { runCapture, runCheckedCapture } from "../../scripts/lib/exec.mjs";
import { fixturesDir, rootDir } from "../../scripts/lib/paths.mjs";
import {
  delegatedCsrf,
  delegatedInternalHeader,
  responseHeaderSchemaType,
  unexpectedRequestHeader,
} from "./mutations/headers.mjs";
import { dmMarkReadSchema, dmPolicySchema } from "./mutations/dms.mjs";
import { pathParameterFormat, queryParameterPattern } from "./mutations/parameters.mjs";
import {
  apiErrorSchemaShape,
  requestBodyMediaType,
  responseBuilderSuccessSchema,
  serverChannelRequestSchema,
} from "./mutations/schemas.mjs";

const validatorPath = path.join(rootDir, "scripts", "validators", "contract-parity.mjs");
const contractFixturesDir = path.join(fixturesDir, "contract-parity");
const author = {
  name: "OpenCode Fixture",
  email: "fixture@hexrelay.local",
};

const cases = [
  fixture("pass-basic", 0),
  fixture("pass-cookie-actions", 0),
  mutation("pass-delegated-csrf-header", "pass-basic", delegatedCsrf, 0),
  mutation("pass-delegated-internal-auth-header", "pass-basic", delegatedInternalHeader, 0),
  mutation("pass-dm-mark-read-scalar-bounds", "pass-session-auth-security", dmMarkReadSchema, 0),
  mutation("pass-path-parameter-format", "pass-basic", pathParameterFormat, 0),
  mutation("pass-response-header-schema-ref", "pass-cookie-actions", responseHeaderSchemaType, 0),
  fixture("pass-request-body-component", 0),
  fixture("pass-request-schema-alias", 0),
  fixture("pass-response-schema-alias", 0),
  fixture("pass-session-auth-security", 0),
  fixture("pass-server-channel-example-status", 0),
  mutation("pass-dm-policy-enum-domain", "pass-session-auth-security", dmPolicySchema, 0),
  fixture("fail-cookie-actions", 1, "issue:hexrelay_csrf"),
  mutation(
    "fail-api-error-schema-shape",
    "pass-basic",
    apiErrorSchemaShape,
    1,
    "`ApiError` must require fields [code, message] but documents [code]",
  ),
  fixture(
    "fail-csrf-header-semantics",
    1,
    "enforces CSRF header `x-csrf-token` as type `string` at runtime but documents `integer`",
  ),
  fixture("fail-discovery-query-semantics", 1, "default:global"),
  fixture("fail-dm-control-example", 1, "dm_policy_invalid"),
  fixture(
    "fail-error-response-schema",
    1,
    "can return HTTP 400 with ApiError at runtime but documents schema `FriendRequestRecord` instead of `ApiError`",
  ),
  fixture("fail-fanout-example", 1, "fanout_invalid"),
  fixture("fail-helper-auth-401", 1, "can return HTTP 401 at runtime via direct unauthorized emitters or local failure helpers"),
  fixture("fail-internal-auth-401", 1, "requires internal-token auth at runtime but is missing a 401 response"),
  fixture("fail-internal-auth-header", 1, "x-hexrelay-internal-token"),
  fixture(
    "fail-internal-auth-header-semantics",
    1,
    "requires request header `x-hexrelay-internal-token` at runtime but it is not marked required",
  ),
  fixture("fail-internal-auth-security", 1, "should not declare session security schemes"),
  fixture("fail-internal-auth-example", 1, "internal_token_invalid"),
  fixture("fail-invite-create-example", 1, "invite_invalid"),
  fixture("fail-missing-csrf-header", 1, "missing the CsrfTokenHeader parameter"),
  fixture("fail-missing-request-body", 1, "missing requestBody"),
  fixture("fail-nonauth-helper-500", 1, "local helper/delegate flows but is missing a 500 response"),
  fixture("fail-no-content-success-schema", 1, "returns HTTP 204 without a JSON success body"),
  mutation(
    "fail-path-parameter-format",
    "pass-basic",
    pathParameterFormat,
    1,
    "uses path parameter `request_id` with format `uuid` at runtime but documents `<none>`",
  ),
  fixture("fail-path-parameter-semantics", 1, "uses path parameter `request_id` as type `string` at runtime but documents `integer`"),
  fixture(
    "fail-public-auth-security",
    1,
    "GET /health documents security schemes [BearerAuth, CookieAuth] but runtime does not require session or internal-token auth",
  ),
  mutation(
    "fail-query-parameter-pattern",
    "pass-basic",
    queryParameterPattern,
    1,
    "uses query parameter `identity_id` with pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`",
  ),
  fixture("fail-request-body-required", 1, "requestBody is not marked required"),
  mutation(
    "fail-request-body-media-type",
    "pass-basic",
    requestBodyMediaType,
    1,
    "accepts JSON request bodies at runtime but documents request media types [application/json, text/plain] instead of [application/json]",
  ),
  fixture("fail-request-schema-ref-direct", 1, "FriendRequestCreateRequest"),
  fixture("fail-request-schema-ref-alias", 1, "FriendRequestCreateRequest"),
  fixture("fail-rest-schema-field-types", 1, "uses request schema `AuthVerifyRequest` field `signature` as type `string` at runtime but documents `integer`"),
  fixture(
    "fail-rest-schema-array-item-ref",
    1,
    "returns schema `DmFanoutCatchUpResponse` field `items` array items as schema `DmFanoutCatchUpItem` at runtime but documents `FriendRequestPage`",
  ),
  mutation(
    "fail-dm-policy-update-inbound-enum-domain",
    "pass-session-auth-security",
    dmPolicySchema,
    1,
    "uses request schema `DmPolicyUpdate` field `inbound_policy` enum [anyone, friends_only, same_server] at runtime but documents [friends_only, same_server]",
  ),
  mutation(
    "fail-dm-policy-response-offline-mode-enum-domain",
    "pass-session-auth-security",
    dmPolicySchema,
    1,
    "returns schema `DmPolicy` field `offline_delivery_mode` enum [encrypted_envelope_catchup] at runtime but documents [<none>]",
  ),
  mutation(
    "fail-dm-mark-read-scalar-bounds",
    "pass-session-auth-security",
    dmMarkReadSchema,
    1,
    "uses request schema `DmThreadMarkReadRequest` field `last_read_seq` minimum `0` at runtime but documents `<none>`",
  ),
  mutation(
    "fail-dm-mark-read-response-last-read-scalar-bound",
    "pass-session-auth-security",
    dmMarkReadSchema,
    1,
    "returns schema `DmThreadMarkReadResponse` field `last_read_seq` minimum `0` at runtime but documents `<none>`",
  ),
  mutation(
    "fail-dm-mark-read-response-unread-scalar-bound",
    "pass-session-auth-security",
    dmMarkReadSchema,
    1,
    "returns schema `DmThreadMarkReadResponse` field `unread` minimum `0` at runtime but documents `<none>`",
  ),
  mutation(
    "fail-rest-schema-array-item-pattern",
    "pass-server-channel-example-status",
    serverChannelRequestSchema,
    1,
    "uses request schema `ServerChannelMessageCreateRequest` field `mention_identity_ids` array items pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`",
  ),
  fixture(
    "fail-rest-schema-date-time-format",
    1,
    "returns schema `AuthVerifyResponse` field `expires_at` format `date-time` at runtime but documents `<none>`",
  ),
  fixture("fail-rest-schema-nullable-field", 1, "uses request schema `DmFanoutCatchUpRequest` field `cursor` nullable `true` at runtime but documents `false`"),
  fixture("fail-rest-schema-scalar-bounds", 1, "uses request schema `DmFanoutCatchUpRequest` field `limit` maximum `100` at runtime but documents `50`"),
  fixture("fail-rest-schema-enum-domain", 1, "returns schema `DmFanoutCatchUpResponse` field `status` enum [blocked, ready] at runtime but documents [ready]"),
  mutation(
    "fail-rest-schema-serde-default-required",
    "pass-server-channel-example-status",
    serverChannelRequestSchema,
    1,
    "uses request schema `ServerChannelMessageCreateRequest` with required fields [content] at runtime but documents [content, mention_identity_ids]",
  ),
  fixture(
    "fail-rest-schema-string-pattern",
    1,
    "uses request schema `AuthVerifyRequest` field `identity_id` pattern `^[A-Za-z0-9_-]{3,64}$` at runtime but documents `<none>`",
  ),
  fixture(
    "fail-rest-schema-nested-item-field-type",
    1,
    "returns schema `DmFanoutCatchUpResponse` field `items` array items reference schema `DmFanoutCatchUpItem` field `ciphertext` as type `string` at runtime but documents `integer`",
  ),
  fixture(
    "fail-rest-schema-required-fields",
    1,
    "uses request schema `AuthVerifyRequest` with required fields [challenge_id, identity_id, signature] at runtime but documents [challenge_id, identity_id]",
  ),
  fixture("fail-realtime-error-envelope-semantics", 1, "Realtime runtime event `error` uses data fields [code, message] but documents [code]"),
  fixture("fail-realtime-envelope-semantics", 1, "Realtime runtime event `realtime.connected` uses data fields [state] but documents [status]"),
  fixture(
    "fail-realtime-signal-envelope-semantics",
    1,
    "Realtime runtime event `call.signal.offer` uses data fields [call_id, from_identity_id, sdp_offer, to_identity_id] but documents [call_id, from_identity_id, to_identity_id]",
  ),
  fixture(
    "fail-realtime-signaling-semantics",
    1,
    "Realtime runtime event `call.signal.offer` requires from_identity_id/session-identity parity at runtime but does not require it",
  ),
  fixture("fail-response-header", 1, "returns response header `Set-Cookie` for HTTP 200 at runtime but is missing it"),
  mutation(
    "fail-response-header-schema-type",
    "pass-cookie-actions",
    responseHeaderSchemaType,
    1,
    "returns response header `Set-Cookie` for HTTP 200 as type `string` at runtime but documents `integer`",
  ),
  mutation(
    "fail-response-builder-success-schema",
    "pass-cookie-actions",
    responseBuilderSuccessSchema,
    1,
    "POST /dev/testing/sessions returns response schema `TestingSessionCreateResponse` for HTTP 200 at runtime but documents `AuthVerifyResponse`",
  ),
  fixture("fail-response-schema-ref", 1, "PresenceWatcherListResponse"),
  fixture("fail-server-channel-example-status", 1, "missing tracked HTTP 400 route-level error examples for ApiError codes [reply_target_invalid]"),
  fixture("fail-session-auth-401", 1, "missing a 401 response"),
  fixture("fail-session-auth-security", 1, "documents security schemes [CookieAuth] instead of [BearerAuth, CookieAuth]"),
  fixture("fail-session-auth-500", 1, "missing a 500 response"),
  fixture("fail-success-content", 1, "documents no success schema"),
  fixture("fail-unexpected-request-body", 1, "documents a requestBody but runtime handler has no request-body extractor"),
  mutation("fail-unexpected-csrf-header", "pass-basic", unexpectedRequestHeader, 1, "documents CsrfTokenHeader but runtime does not enforce CSRF"),
  mutation(
    "fail-unexpected-internal-auth-header",
    "pass-basic",
    unexpectedRequestHeader,
    1,
    "documents request header `x-hexrelay-internal-token` but runtime does not require it",
  ),
  mutation(
    "fail-unexpected-internal-auth-header-component",
    "pass-basic",
    unexpectedRequestHeader,
    1,
    "documents request header `x-hexrelay-internal-token` but runtime does not require it",
  ),
  fixture("fail-missing-example", 1, "thread_not_found"),
];

function fixture(name, expectedExit, expectedText = "") {
  return { name, fixtureName: name, expectedExit, expectedText };
}

function mutation(name, fixtureName, mutate, expectedExit, expectedText = "") {
  return { name, fixtureName, mutate, expectedExit, expectedText };
}

async function copyFixture(fixtureName, tempRepo) {
  const fixtureDir = path.join(contractFixturesDir, fixtureName);
  await fs.cp(fixtureDir, tempRepo, { recursive: true });
  await fs.copyFile(path.join(rootDir, ".gitattributes"), path.join(tempRepo, ".gitattributes"));
}

function git(cwd, args) {
  return runCheckedCapture("git", args, { cwd });
}

function commit(cwd, message, extraArgs = []) {
  git(cwd, ["-c", `user.name=${author.name}`, "-c", `user.email=${author.email}`, "commit", ...extraArgs, "-qm", message]);
}

async function prepareRepo(testCase, tempRepo) {
  await copyFixture(testCase.fixtureName, tempRepo);
  git(tempRepo, ["init", "-q"]);
  commit(tempRepo, "base", ["--allow-empty"]);
  if (testCase.mutate) {
    await testCase.mutate(tempRepo, testCase.name);
  }
  git(tempRepo, ["add", "."]);
  commit(tempRepo, "fixture");
}

async function runCase(testCase) {
  const tempRepo = await fs.mkdtemp(path.join(os.tmpdir(), "hexrelay-contract-parity-"));
  try {
    await prepareRepo(testCase, tempRepo);
    const result = runCapture("node", [validatorPath, "HEAD~1", "HEAD"], { cwd: tempRepo });
    const output = `${result.stdout}${result.stderr}`;

    if (result.status !== testCase.expectedExit) {
      throw new Error(`fixture ${testCase.name}: expected exit ${testCase.expectedExit}, got ${result.status}\n${output}`);
    }

    if (testCase.expectedText && !output.includes(testCase.expectedText)) {
      throw new Error(`fixture ${testCase.name}: expected output to contain ${testCase.expectedText}\n${output}`);
    }
  } finally {
    await fs.rm(tempRepo, { recursive: true, force: true });
  }
}

function printHelp() {
  console.log("Usage: node tests/contract-parity/run.mjs");
}

try {
  if (process.argv.includes("--help") || process.argv.includes("-h")) {
    printHelp();
    process.exit(0);
  }

  for (const testCase of cases) {
    console.log(`[contract-parity-test] ${testCase.name}`);
    await runCase(testCase);
  }
  console.log("[contract-parity-test] Fixture regressions passed");
} catch (error) {
  console.error(`::error::${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}
