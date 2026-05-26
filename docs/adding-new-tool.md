# Adding a New Tool to the Lago MCP

> Canonical runbook for shipping a new tool to the Lago AI Assistant (e.g., `list_fees`, `get_credit_note`, `cancel_subscription`). The work spans three repos and two crates.io publishes, plus a manual Mistral console paste.

**For Claude**: when a user asks to "add a new MCP tool", "expose a Lago API to the assistant", "create a new MCP function", or anything similar — follow this document end-to-end. Don't shortcut the phases; each one depends on the previous being live before the next can build.

**Real example to reference**: `list_fees` / `get_fee` shipped via PRs [`getlago/lago-rust-client#44`](https://github.com/getlago/lago-rust-client/pull/44), [`#45`](https://github.com/getlago/lago-rust-client/pull/45), [`#47`](https://github.com/getlago/lago-rust-client/pull/47), and [`getlago/lago-agent-toolkit#51`](https://github.com/getlago/lago-agent-toolkit/pull/51). Use those diffs as concrete templates.

---

## Architecture (what you're touching)

```
┌────────────────────────┐    ┌─────────────────────────┐    ┌────────────────────────┐
│ lago-rust-client       │ →  │ lago-agent-toolkit      │ →  │ Mistral agent console  │
│ (Rust SDK, two crates) │    │ (Rust MCP server)       │    │ (function schemas)     │
│ lago-types, lago-client│    │ lago-mcp-server         │    │ Lago Billing Assistant │
└────────────────────────┘    └─────────────────────────┘    └────────────────────────┘
        ↓ publishes                  ↓ Docker image                  ↓ pasted JSON
   crates.io                    Deployed to staging/prod         Picked by Mistral LLM
                                                                              ↓
                                                            ┌─────────────────────────┐
                                                            │ lago-api (Ruby)         │
                                                            │ orchestration layer:    │
                                                            │ Mistral ↔ MCP server    │
                                                            └─────────────────────────┘
```

Two crates.io publishes (`lago-types`, then `lago-client`), one Docker image rollout, and one manual schema paste sit between writing the code and the assistant actually using the tool. The Lago API Ruby code already routes tool calls dynamically — no allowlist update needed there.

---

## Prerequisites (one-time setup)

- **Rust toolchain**: `brew install mise && mise install` from inside `lago-rust-client/` or this repo. Both repos use `mise` to install Rust (the `mise.toml` files specify `rust = "latest"`, not a pinned version — first install will pull the current stable channel).
- **crates.io account**: sign in at https://crates.io with GitHub, verify your email at https://crates.io/settings/profile, then `cargo login <token>` from https://crates.io/settings/tokens. Note that `~/.cargo/credentials.toml` can be overwritten by `cargo login` — if you already use a private registry, back it up first.
- **Crate ownership** for both `lago-types` and `lago-client`. Check current owners with `cargo owner --list lago-types` / `cargo owner --list lago-client`. To get added: ping the current owner with your crates.io username (which matches your GitHub username). Then **accept the pending invitation** at https://crates.io/me/pending-invites — until you do, you can't publish.
- **Mistral console access**: https://console.mistral.ai/, with access to the `Lago Billing Assistant` and `Lago Billing Assistant Staging` agents.
- **Staging Lago API key**: ask whoever owns staging credentials (typically platform/SRE) — needed for local MCP smoke testing and end-to-end UI testing. Production keys are not needed until the very end and are gated separately.

---

## The six-phase workflow

Each phase is concrete work. Don't start the next phase before the previous one is fully live (merged + published or merged + deployed, depending on the phase).

### Phase 1: Add SDK types in `lago-rust-client`

Goal: add the Rust data shapes the new tool consumes — filters, request, response, and possibly a new model.

**Steps**:

1. Branch from `main`:
   ```bash
   cd ~/Documents/GitHub/lago-rust-client
   git checkout main && git pull
   git checkout -b feat/<resource>-types
   ```

2. Add the per-resource files for a new resource `widget` (substitute your actual resource name):
   - `lago-types/src/filters/widget.rs` — `WidgetFilters` struct implementing the `ListFilters` trait
   - `lago-types/src/requests/widget.rs` — `ListWidgetsRequest`, `GetWidgetRequest`, and any other request types
   - `lago-types/src/responses/widget.rs` — `ListWidgetsResponse`, `GetWidgetResponse`
   - **Models** — two cases:
     - **If the type is brand new**: create `lago-types/src/models/widget.rs` with the `Widget` struct
     - **If the type is already nested under an existing model**: just edit the existing file. For example, `list_fees` added a new `FeeType` enum directly into `models/invoice.rs` because `Fee`, `FeePaymentStatus`, and `FeeItem` were already there. No new file or registration needed.

3. Register the modules:
   - `pub mod widget;` in `lago-types/src/filters.rs`
   - `pub mod widget;` in `lago-types/src/requests.rs`
   - `pub mod widget;` in `lago-types/src/responses.rs`
   - **Only if you created a new models file**: `pub mod widget;` in `lago-types/src/models.rs`

4. Verify locally — run **all six** CI steps to match what GitHub Actions runs (the CI workflow is `.github/workflows/ci.yml`):
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-features -- -D warnings
   cargo test --all-features
   cargo build --release --all-features
   (cd lago-types && cargo publish --dry-run)
   (cd lago-client && cargo publish --dry-run)
   ```
   All six must pass. If you only run the first three, you risk passing locally and failing CI on the dry-runs.

5. Open a PR (title format: `feat: add <resource> endpoint type support`). Wait for review + merge.

> **Reference**: `list_fees` types landed in [`lago-rust-client` PR #44](https://github.com/getlago/lago-rust-client/pull/44).

### Phase 2: Publish `lago-types` to crates.io

Goal: make the new types available to downstream consumers via crates.io. The merge in Phase 1 does NOT auto-publish — someone has to run `cargo publish` manually.

**Steps**:

1. Open a small bump-only PR to bump the `lago-types` version:
   ```bash
   git checkout main && git pull
   git checkout -b bump-lago-types-X.Y.Z
   # edit lago-types/Cargo.toml: bump version (use patch unless breaking changes)
   cargo build --all-targets    # regenerates Cargo.lock
   git add lago-types/Cargo.toml Cargo.lock
   git commit -m "chore: bump lago-types to X.Y.Z"
   git push -u origin bump-lago-types-X.Y.Z
   gh pr create --title "chore: bump lago-types to X.Y.Z" --body "$(cat <<'EOF'
   Bumps lago-types to X.Y.Z. Required so downstream PRs (lago-client / lago-agent-toolkit) can depend on the new types currently sitting unpublished on main.

   After merge, run \`cargo publish\` from \`lago-types/\` on main.
   EOF
   )"
   ```

   **Version-bump policy**: use a patch version (`0.1.22` → `0.1.23`) for additive changes (new types, new fields with sensible defaults). Use a minor version (`0.1.x` → `0.2.0`) for breaking changes (removed types, changed required fields, renamed exports). Since the crate is at `0.1.x`, both patch and minor are signals; no need to invoke major.

2. After merge, publish from `main`:
   ```bash
   git checkout main && git pull
   cd lago-types
   cargo publish --dry-run    # sanity check
   cargo publish              # actual upload
   ```

   `cargo publish` is **irreversible** — once a version is on crates.io you can't overwrite it. You can `cargo yank` to mark a version as not-for-new-use, but the only way to fix a mistake is to publish a new bump.

3. Verify the publish is live (can take ~30 seconds to propagate):
   ```bash
   curl -s https://crates.io/api/v1/crates/lago-types | python3 -c "import sys,json; print(json.load(sys.stdin)['crate']['max_version'])"
   ```

> **Reference**: `lago-types 0.1.23` was bumped via [`lago-rust-client` PR #45](https://github.com/getlago/lago-rust-client/pull/45) and published.

### Phase 3: Add SDK client methods in `lago-rust-client` + publish

Goal: implement the actual `LagoClient::list_widgets` / `LagoClient::get_widget` HTTP methods on top of the types from Phase 1, and publish the new `lago-client` version.

**Steps**:

1. Branch from updated `main`:
   ```bash
   cd ~/Documents/GitHub/lago-rust-client
   git checkout main && git pull
   git checkout -b feat/<resource>-client
   ```

2. Add the query module `lago-client/src/queries/widget.rs`. Use [`update_billable_metric`](https://github.com/getlago/lago-rust-client/pull/38) as the canonical reference for the `Url::parse` + `LagoError::Configuration` mapping pattern. Apply it to **both** list and detail methods:

   ```rust
   use url::Url;
   use lago_types::{error::{LagoError, Result}, ...};

   impl LagoClient {
       pub async fn list_widgets(&self, request: ListWidgetsRequest) -> Result<ListWidgetsResponse> {
           let region = self.config.region()?;
           let url = Url::parse(&format!("{}/widgets", region.endpoint()))
               .map_err(|e| LagoError::Configuration(format!("Invalid URL: {e}")))?;
           // ... append query params from request.to_query_params() ...
           self.make_request("GET", url.as_str(), None::<&()>).await
       }

       pub async fn get_widget(&self, request: GetWidgetRequest) -> Result<GetWidgetResponse> {
           let region = self.config.region()?;
           let url = Url::parse(&format!("{}/widgets/{}", region.endpoint(), request.widget_id))
               .map_err(|e| LagoError::Configuration(format!("Invalid URL: {e}")))?;
           self.make_request("GET", url.as_str(), None::<&()>).await
       }
   }
   ```

3. Register the module: `pub mod widget;` in `lago-client/src/queries.rs`.

4. Add `lago-client/examples/widget.rs` — a runnable example demonstrating both methods. Register the example in `lago-client/Cargo.toml`:
   ```toml
   [[example]]
   name = "widget"
   path = "examples/widget.rs"
   ```

5. Bump versions in `lago-client/Cargo.toml`:
   - Bump `version = "..."` for lago-client itself
   - Bump `lago-types = "X.Y.Z"` to the version published in Phase 2

6. Verify all six CI steps locally (same list as Phase 1 — `fmt`, `clippy`, `test`, release build, and both dry-runs). The `lago-client` dry-run will now fetch the new `lago-types` from crates.io — this is the moment that confirms Phase 2 actually published.

7. Open PR (title: `feat(client): add list_widgets and get_widget methods`). Wait for merge.

8. Publish:
   ```bash
   git checkout main && git pull
   cd lago-client
   cargo publish
   ```

   Verify with the same crates.io curl as Phase 2 step 3.

> **Reference**: `LagoClient::list_fees` / `get_fee` landed in [`lago-rust-client` PR #47](https://github.com/getlago/lago-rust-client/pull/47).

### Phase 4: Add MCP tool in `lago-agent-toolkit`

Goal: wire the SDK methods into the MCP server as named tools the LLM can call.

**Steps**:

1. Branch from `main`:
   ```bash
   cd ~/Documents/GitHub/lago-agent-toolkit
   git checkout main && git pull
   git checkout -b feat/<tool-name>-tool
   ```

2. Bump deps in `mcp/Cargo.toml` to the versions just published:
   ```toml
   lago-client = "<version from Phase 3>"
   lago-types = "<version from Phase 2>"
   ```

3. Add `mcp/src/tools/<resource>.rs`. Mirror `mcp/src/tools/invoice.rs` as the canonical pattern. The file contains:
   - **Args structs** with `#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]`. Each arg field gets a `///` doc comment that becomes the JSON Schema `description` the LLM reads. Be precise — these descriptions drive correct LLM behavior.
   - **Service struct** (e.g., `WidgetService`) with handler methods that take `Parameters<WidgetArgs>` + `RequestContext<RoleServer>` and call the corresponding `LagoClient` methods.

4. Register the module: `pub mod widget;` in `mcp/src/tools.rs`.

5. Wire into `mcp/src/server.rs`:
   - `use crate::tools::widget::WidgetService;` at the top
   - Add `widget_service: WidgetService,` to the `LagoMcpServer` struct
   - Instantiate `let widget_service = WidgetService::new();` in `new()` and add to the `Self { ... }` literal
   - Add `#[tool(description = "...")]` methods in the `#[tool_router] impl` block. The description string is **LLM-facing product copy** — write it like product copy. Look at the existing `list_fees` description for a strong template that explicitly mentions use cases:
     ```rust
     #[tool(
         description = "List widgets from Lago with optional filtering by ... . \
             Use this to access widget-level data for <specific use case>. \
             For <variant scenario>, filter by `<field>`."
     )]
     ```

6. Update root `README.md` with a new section under "Available Tools" describing the new tool(s) in user-facing terms.

7. **Set up the local dev environment** for smoke testing. The `mcp/.env.development.example` only has `MISTRAL_AGENT_ID` and `MISTRAL_API_KEY`. You need to create `mcp/.env.development` (gitignored — won't be committed) with two additional keys:
   ```bash
   cat > mcp/.env.development << 'EOF'
   LAGO_API_KEY=<your staging Lago API key>
   LAGO_API_URL=https://api.staging.getlago.com/api/v1
   MISTRAL_AGENT_ID=<staging agent ID>
   MISTRAL_API_KEY=<staging Mistral key>
   EOF
   chmod 600 mcp/.env.development
   ```

8. Verify locally:
   ```bash
   cd mcp
   cargo build --all-targets
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```

9. Smoke test by running the MCP server locally and exercising the new tool end-to-end:
   ```bash
   cd ~/Documents/GitHub/lago-agent-toolkit/mcp
   set -a && source ./.env.development && set +a
   cargo run -- sse --port 3000
   ```

   In another terminal, either:
   - **Easy path**: use [MCP Inspector](https://github.com/modelcontextprotocol/inspector) — `npx @modelcontextprotocol/inspector`, connect to `http://127.0.0.1:3000/mcp` with `X-LAGO-API-KEY` header, click through tools.
   - **Scriptable path**: use the full session-ID curl dance:
     ```bash
     LAGO_API_KEY=<your key>
     BASE=http://127.0.0.1:3000/mcp

     # Initialize and capture session ID
     SID=$(curl -s -X POST $BASE \
       -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
       -H "X-LAGO-API-KEY: $LAGO_API_KEY" \
       -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0.1"}},"id":1}' \
       -D - -o /dev/null | grep -i '^mcp-session-id:' | awk '{print $2}' | tr -d '\r')

     # Send initialized notification (required by MCP protocol)
     curl -s -X POST $BASE \
       -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
       -H "X-LAGO-API-KEY: $LAGO_API_KEY" -H "Mcp-Session-Id: $SID" \
       -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null

     # Confirm the new tool is advertised
     curl -s -X POST $BASE \
       -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
       -H "X-LAGO-API-KEY: $LAGO_API_KEY" -H "Mcp-Session-Id: $SID" \
       -d '{"jsonrpc":"2.0","method":"tools/list","id":2}' \
       | sed -n 's/^data: //p' | python3 -c "import sys,json; d=json.load(sys.stdin); print([t['name'] for t in d['result']['tools'] if 'widget' in t['name']])"

     # Call the tool
     curl -s -X POST $BASE \
       -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
       -H "X-LAGO-API-KEY: $LAGO_API_KEY" -H "Mcp-Session-Id: $SID" \
       -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_widgets","arguments":{"per_page":3}},"id":3}' \
       | sed -n 's/^data: //p' | python3 -m json.tool
     ```

   Confirm real data flows back. Also test the **failure modes**: call with a bad argument (e.g., wrong UUID format) and verify the assistant returns a sensible error.

   > **Port note**: if port 3000 is already used by Vite/Rails/anything else, swap to another port (e.g., `--port 3001`).

10. Open PR (title: `feat(mcp): add <tool_name> tool`). Wait for review + merge.

> **Reference**: `list_fees` / `get_fee` MCP tools landed in [`lago-agent-toolkit` PR #51](https://github.com/getlago/lago-agent-toolkit/pull/51).

### Phase 5: Deploy MCP staging + sync Mistral staging

Goal: get the new MCP server image running in staging, and teach the staging Mistral agent about the new tools.

**Steps**:

1. **MCP staging deploy**. The PR merge auto-triggers `.github/workflows/mcp-docker-build.yml`, which pushes a new `getlago/lago-mcp-server` image (typically tagged `:latest` plus a SHA tag). The deploy to staging is **separate** — usually owned by platform/SRE. Confirm in the relevant Slack channel (often `#platform` or `#infra` for Lago) that the new image is rolled out. Roll back is by redeploying the previous SHA tag, not by deleting the new one (`:latest` is mutable, so always reference SHAs for rollback safety).

2. Verify the staging MCP server has the new tool. Either:
   - Ask the assistant a question that should trigger the tool — if you get `Tool '<name>' not found`, the deploy hasn't happened yet.
   - Curl the staging MCP server's `tools/list` directly (replace `<staging-mcp-url>` with the actual URL — ask whoever owns staging if you don't know it). Use the session-ID curl dance from Phase 4 step 9.

3. **Mistral staging sync**. The MCP server is now serving the new tools, but the Mistral agent doesn't know about them until the schemas are pasted into its function list.

   a. Get the schema JSON from the locally-running MCP server (cleanest source). This reuses `$LAGO_API_KEY` and `$SID` from Phase 4 step 9 — if you've opened a fresh shell, re-run the `initialize` + `notifications/initialized` curls from that step to repopulate them before running the snippet below.
      ```bash
      # Run a local MCP server (Phase 4 step 9), then:
      curl -s -X POST http://127.0.0.1:3000/mcp \
        -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" \
        -H "X-LAGO-API-KEY: $LAGO_API_KEY" -H "Mcp-Session-Id: $SID" \
        -d '{"jsonrpc":"2.0","method":"tools/list","id":2}' \
        | sed -n 's/^data: //p' \
        | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps([t for t in d['result']['tools'] if t['name'] in ('list_widgets','get_widget')], indent=2))" \
        > ~/widget-tool-schemas.json
      ```

   b. Open the **Staging** agent in [Mistral console](https://console.mistral.ai/) → **+ Add** under *Functions*.

   c. For each new tool, paste three things from the JSON:
      - **Name** field → the `name` value (e.g., `list_widgets`)
      - **Description** field → the `description` value
      - **JSON Schema** field → the `inputSchema` value (the inner object, not the whole tool entry)

   d. Enable **Strict** mode. (Strict tells Mistral to validate the LLM's tool arguments against the JSON Schema before sending — required fields enforced, unknown fields rejected. Always enable for new tools.)

   e. The `"format": "int32"` warning Mistral may show for integer fields is harmless. It's an OpenAPI-style format hint that JSON Schema draft-07 doesn't enforce. Every existing tool in the agent has the same warning. Save anyway.

   f. Click **Save Agent Changes** at the top right of the console. This bumps the agent version (e.g., v17 → v18).

4. **System prompt check**. If the new tool requires new behavioral rules — e.g., "always ask for confirmation before doing X", or "interpret 'recent' as last 30 days" — the system prompt may also need updating in the same agent config. The system prompt is the text in the **Instructions** section. Edit if needed, then save again.

5. **End-to-end test via the staging Lago app UI**. The preview pane in Mistral console only shows LLM tool selection, not execution. The real test runs through the staging Lago app where the Ruby orchestration in `lago-api` connects Mistral planning to MCP execution.
   - Open a fresh AI conversation in the staging Lago app
   - Ask a representative prompt (e.g., for `list_widgets`: "show me 3 recent widgets")
   - Confirm real data comes back

   **Known bug**: if the tool fails for any reason (`ToolNotFoundError`, network error, MCP 500), the entire conversation locks up because `lago-api`'s error handler doesn't send a synthetic function result back to Mistral. Workaround: start a fresh conversation. The underlying bug is tracked separately and is not a regression from new tools.

### Phase 6: Production rollout

Goal: replicate the staging deploy + Mistral sync for production.

**Steps**:

1. **Production MCP deploy**. Same image, separate rollout (likely a different env in your deploy tooling). Coordinate with platform/SRE.

2. **Production Mistral sync**. Repeat Phase 5 step 3 on the **production** Mistral agent (the one without "Staging" in the name). Paste the same JSON schemas you used for staging. Save.

3. **End-to-end test in production**. Same prompt, production UI. Verify real data.

4. **Watch for an hour**. Tail the relevant logs (lago-api stream service errors, MCP server logs, Mistral API rate-limit responses) for any anomalies in the first usage.

---

## Gotchas (from real shipping experience)

1. **Publish-before-merge is mandatory.** crates.io is the source of truth for cross-crate deps in CI. If `lago-client/Cargo.toml` says `lago-types = "0.1.23"` and `0.1.23` isn't on crates.io yet, `cargo publish --dry-run` for lago-client fails. Always publish the upstream crate *before* opening the downstream PR.

2. **Don't add `[patch.crates-io]` to a PR.** It's a great local-dev tool (lets you build cross-crate without publishing) but CI's `cargo publish --dry-run` ignores workspace patches. Keep workspace patches out of merged commits.

3. **Don't add `path = "../foo"` to a published crate's `Cargo.toml`.** Same reason — against the maintainer's convention; the publish-first workflow doesn't need it.

4. **Match the snake_case convention for enums in JSON Schema.** `schemars` derives schemas using `#[serde(rename_all = "snake_case")]`. For Rust variants with PascalCase compound names like `FeeType::AddOn`, the naive serialization `format!("{:?}", variant).to_lowercase()` produces `"addon"` — which the Lago API rejects (it expects `"add_on"`). Use an explicit match helper instead:
   ```rust
   fn fee_type_param(ft: &FeeType) -> &'static str {
       match ft {
           FeeType::Charge => "charge",
           FeeType::AddOn => "add_on",
           FeeType::Subscription => "subscription",
           FeeType::Credit => "credit",
           FeeType::Commitment => "commitment",
       }
   }
   ```
   See `lago-types/src/filters/fee.rs` for the actual implementation in production.

5. **The `format: "int32"` warning in Mistral is harmless.** OpenAPI-style integer hint that JSON Schema draft-07 doesn't enforce. Every existing tool in the agent has the same warning (it's emitted by `schemars` for all `i32` fields). Save anyway.

6. **GitHub Desktop / IDE auto-pull can hijack force-push cleanup.** If you trim commits then leave the branch idle, an auto-pull may merge the stale `origin/<branch>` back in. After any `git rebase` that rewrites history, push promptly (with `--force-with-lease`) before any background tool fetches.

7. **Tool failures lock conversations (known lago-api bug).** If any tool call fails (`ToolNotFoundError`, network timeout, MCP 500), the entire `AiConversation` becomes unrecoverable because the Ruby error handler doesn't send a synthetic function result back to Mistral. Workaround: start a fresh conversation. Tracked as a separate Linear issue — the fix is in `lib/lago_mcp_client/lago_mcp_client/run_context.rb` to catch per-tool exceptions and emit an error result.

8. **crates.io is append-only, and rate-limits consecutive publishes.** Two related constraints:
   - **Versions are immutable.** Once `0.1.23` is published you can't overwrite it. If you find a bug after publish, bump to `0.1.24` and publish again. Use `cargo yank` to discourage use of a known-bad version.
   - **There's a publish throttle.** crates.io rejects rapid back-to-back publishes from the same account (the limit is roughly one publish per crate per ~minute). Not a problem in this workflow because Phase 3 has a PR review between the two publishes, but worth knowing if you ever try to script both in one go.

9. **`cargo publish --dry-run` doesn't catch upstream-pin mismatch on the real registry.** A clean dry-run can still fail at actual publish if the upstream just landed but didn't propagate yet. Wait ~30 seconds after publishing upstream before opening the downstream PR.

10. **Strict mode in Mistral changes required fields enforcement.** With Strict on, the LLM must produce arguments that pass JSON Schema validation (required fields present, no unknown fields, correct types). With Strict off, the LLM can produce malformed args that hit the handler and fail there instead. Always enable Strict for new tools.

---

## On destructive tools

Some shipping tools are destructive — they mutate Lago data. Examples already in production: `void_invoice`, `delete_subscription`, `delete_coupon`, `delete_plan`, `update_*` variants. These work today through the LLM, but **there is no code-level safety scaffolding** — no `confirmation_token`, no two-phase commit, no per-tool capability gating. The "confirm before destructive action" rule lives purely in the Mistral agent's system prompt and is therefore vulnerable to prompt injection or model regression.

If your new tool mutates data:
- It can ship through this same workflow (the existing destructive tools demonstrate this is possible)
- But raise the lack of safety scaffolding with the team first — adding another mutating tool without a plan to fix the architectural gap is a deliberate decision worth discussing
- At minimum, the tool's description should be explicit about destructiveness ("Deletes the widget permanently. This action cannot be undone.")

For a structural fix, two patterns to consider:
- A `confirmation_token` arg required on every mutating tool, validated by the MCP server against a token issued by a prior preview tool call
- A capability flag in `mcp/Cargo.toml` (e.g., `--no-default-features` to disable destructive tools) gated on the org's plan tier

Neither exists yet. If you ship a new destructive tool, treat the safety burden as carried entirely by the system prompt — which means the system prompt may also need updating.

---

## Verification checklist (paste into the PR descriptions)

```
- [ ] PR #1 (lago-types changes) opened, CI green (all 6 steps), merged
- [ ] lago-types vX.Y.Z published to crates.io
- [ ] PR #2 (lago-client changes) opened, CI green, merged
- [ ] lago-client vA.B.C published to crates.io
- [ ] PR #3 (lago-agent-toolkit MCP tool) opened, CI green, merged
- [ ] MCP Docker image deployed to staging
- [ ] tools/list against staging MCP includes the new tool
- [ ] Schema pasted into staging Mistral agent, saved
- [ ] System prompt updated in staging Mistral agent (if needed)
- [ ] End-to-end test in staging Lago app UI returns real data
- [ ] Failure-mode test (bad args → sensible error, no conversation lockup)
- [ ] MCP Docker image deployed to production
- [ ] Schema pasted into production Mistral agent, saved
- [ ] System prompt updated in production Mistral agent (if needed)
- [ ] End-to-end test in production Lago app UI returns real data
- [ ] Watched logs for ~1 hour post-rollout
```

---

## What this guide does NOT cover

- **Safety scaffolding for destructive tools.** The architectural pattern doesn't exist yet. Treat as a known gap, see the "On destructive tools" section above.
- **Catalog drift detection.** There's no CI check that the Mistral agent's function list matches the MCP server's `tools/list`. Manual paste is the only sync mechanism today. A `bin/sync-mistral` script that calls Mistral's Agents API would close this gap and is a known followup.
- **Removing a tool.** This guide adds tools. Removing involves the reverse: remove from MCP server → publish → update Mistral agent → deploy. The rollback path mirrors the deploy path, but specific deprecation policy (when is it safe to remove a tool that customers may depend on?) is a product decision, not covered here.

---

## Quick links

- **Repos**: [`lago-rust-client`](https://github.com/getlago/lago-rust-client), [`lago-agent-toolkit`](https://github.com/getlago/lago-agent-toolkit), [`lago-api`](https://github.com/getlago/lago-api)
- **Mistral console**: https://console.mistral.ai/
- **MCP Inspector** (debugging UI): https://github.com/modelcontextprotocol/inspector
- **crates.io owner management**: `cargo owner --list <crate>` / `cargo owner --add <user> --crate <crate>`
- **Reference PR chain (fees)**: lago-rust-client [#44](https://github.com/getlago/lago-rust-client/pull/44) → [#45](https://github.com/getlago/lago-rust-client/pull/45) → [#47](https://github.com/getlago/lago-rust-client/pull/47) → lago-agent-toolkit [#51](https://github.com/getlago/lago-agent-toolkit/pull/51)
