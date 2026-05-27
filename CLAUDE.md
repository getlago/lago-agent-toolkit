# Repository context for Claude

This repository (`getlago/lago-agent-toolkit`) is the Rust MCP server (`lago-mcp-server`) that exposes Lago billing operations as tools for AI agents. It's shipped as a Docker image (`getlago/lago-mcp-server`) consumed by both Claude Desktop and the Lago AI Assistant (via Mistral + a Ruby orchestration layer in `getlago/lago-api`).

## When the user wants to add a new MCP tool

**Read [`docs/adding-new-tool.md`](docs/adding-new-tool.md) first, in full, before producing any code or recommendations.**

That document is the canonical runbook for the cross-repo workflow: it covers SDK changes in `lago-rust-client`, two crates.io publishes, MCP server changes in this repo, Docker deploy, and the Mistral console schema paste. It captures all the gotchas from real shipping experience.

Trigger phrases that should make you consult this doc:

- "add a new MCP tool"
- "add a tool to the Lago AI assistant"
- "expose a Lago API endpoint to Claude"
- "create a new MCP function"
- "wire a new endpoint into the assistant"
- anything resembling "make the assistant able to do X" where X corresponds to a Lago API operation

Do not improvise from your training data — the cross-repo dependency chain has specific failure modes that aren't obvious without the doc.

**Heads up**: Phases 1 and 3 of the runbook require a local checkout of [`getlago/lago-rust-client`](https://github.com/getlago/lago-rust-client). If the user doesn't have it cloned anywhere, ask them to clone it before starting and confirm the path you'll use in the workflow.

## Repository layout

- `mcp/` — the Rust MCP server crate (`lago-mcp-server`)
- `mcp/src/tools/` — one file per resource (`invoice.rs`, `fee.rs`, `customer.rs`, ...) — each contains args structs (with `#[derive(schemars::JsonSchema)]`) and a service struct with handler methods
- `mcp/src/server.rs` — `LagoMcpServer` aggregator with `#[tool(description=...)]` registrations in a `#[tool_router]` impl block
- `mcp/src/tools.rs` — module registration + shared `LagoClient` factory (`create_lago_client`) + result helpers (`success_result`, `error_result`)
- `mcp/src/main.rs` — CLI entry point (stdio or SSE transport via `clap`)
- `example/mcp_client/ruby/` — Ruby reference MCP client; production client lives in `getlago/lago-api/lib/lago_mcp_client/`

## Related repositories

- **`getlago/lago-rust-client`** — Rust SDK with two published crates: `lago-types` (data shapes) and `lago-client` (HTTP client). This MCP server depends on both. Independent versioning on crates.io.
- **`getlago/lago-api`** — Ruby orchestration layer that wraps Mistral's Conversations API + this MCP server for the Lago AI Assistant. Manages `AiConversation` records, streams responses via GraphQL subscriptions, and proxies tool calls to MCP. See `app/services/ai_conversations/stream_service.rb` and `lib/lago_mcp_client/`.
- **Mistral agent console** (https://console.mistral.ai/) — agent configuration including function schemas, system prompt, and model settings. Schemas are pasted by hand from this MCP server's `tools/list` response. Two agents: production (`Lago Billing Assistant`) and staging (`Lago Billing Assistant Staging`).

## Security model (load-bearing — read before suggesting any tool)

Every MCP tool routes through the Lago Rails API via the `X-LAGO-API-KEY` header forwarded on each call. Rails performs both auth (validates the key) and **org scoping** (the key identifies which tenant's data is accessible). This is the **only** trust boundary — the MCP server itself does not enforce org isolation.

When designing or reviewing a new tool, this means:

- **Don't add tools that talk directly to Postgres** or any storage layer that bypasses the Rails API. All data access must go through `lago-client` → Rails.
- **Don't accept a client-supplied org/tenant/customer ID as an authoritative identifier.** Rails infers the org from the API key — any org/tenant identifier in a tool's args should be treated as a filter or selector *within* the caller's org, never as cross-org access.
- **Don't forward the API key anywhere except Rails.** No third-party services, no logging it, no embedding it in tool responses.
- **Don't introduce tool-level capability checks** (e.g., "is this user allowed to void?") as a substitute for Rails-side permissions. If Rails allows the call, the tool allows the call. If you want finer-grained gating, that belongs upstream in Rails, not in the MCP server.

If a proposed tool would violate any of these, stop and raise it with the team before writing code.

## Conventions worth knowing

- **Mirror existing patterns.** When adding a new resource, look at `mcp/src/tools/invoice.rs` for the canonical structure. The codebase consistently uses the same shape across resources — don't innovate.
- **Tool descriptions are LLM-facing product copy.** The `#[tool(description = "...")]` string drives the LLM's tool-selection behavior. Write it like product copy: state what the tool does, when to use it, and any important constraints inline. Look at `list_fees` in `mcp/src/server.rs` for a strong template — it explicitly calls out the MRR use case so the LLM surfaces the tool for revenue questions.
- **Use `Url::parse` with `LagoError::Configuration` mapping** for URL construction in `lago-client` — see `update_billable_metric` in [`lago-rust-client` PR #38](https://github.com/getlago/lago-rust-client/pull/38). Applies to both list and detail endpoints. Match the pattern across the codebase.
- **Schemars derive expects `#[serde(rename_all = "snake_case")]`** on enum types. For multi-word PascalCase variants (e.g., `AddOn`), use explicit `match` helpers to serialize to query params — the naive `format!("{:?}", x).to_lowercase()` would produce `"addon"` instead of the required `"add_on"`.
- **Read-only tools are easy.** Mutating tools (create/update/delete) work but have no architectural safety scaffolding — the "confirm first" rule is LLM-enforced via the system prompt. If adding a new destructive tool, raise the safety gap with the team before shipping. See `docs/adding-new-tool.md` § "On destructive tools".

## Don't do these things

- **Don't add `[patch.crates-io]` or `path = "../foo"` to PRs.** They're useful for local cross-crate dev, but CI's `cargo publish --dry-run` ignores workspace patches and rejects path deps. Keep them out of merged commits.
- **Don't open a downstream PR before the upstream crate is published.** crates.io is the source of truth for cross-crate deps in CI. See the publish-before-merge gotcha in the runbook.
- **Don't skip the local CI parity check.** The CI runs six steps (`fmt`, `clippy`, `test`, `build --release`, two `cargo publish --dry-run`). Run all six locally before pushing — many failures are caught only by the dry-runs.
- **Don't shortcut the Mistral schema paste.** The MCP server can advertise a new tool, but until the schema is in the Mistral agent's function list, the LLM doesn't know it exists. Even an end-to-end PR merge doesn't make a tool usable in the assistant without this manual step.

## Quick sanity checks

If a user asks "is this working?" or "did the deploy land?", these are the cheap checks:

- **Published crate version on crates.io**: `curl -s https://crates.io/api/v1/crates/<crate> | python3 -c "import sys,json; print(json.load(sys.stdin)['crate']['max_version'])"`
- **MCP server advertising a tool**: run `tools/list` via curl or MCP Inspector against the relevant MCP URL — see the runbook for the full session-ID curl dance
- **Mistral agent has a function registered**: open the agent in https://console.mistral.ai/ and scan the Functions list — the total count and the new tool name should both be visible
- **CI status on a recent PR**: `gh pr checks <number> --repo getlago/<repo>`
- **MCP Docker build status**: `gh run list --repo getlago/lago-agent-toolkit --workflow=mcp-docker-build.yml --limit 3`
