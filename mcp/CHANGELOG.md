# Changelog

All notable changes to `lago-mcp-server` are documented here.

## [Unreleased]

### Changed — bump `rmcp` from 0.6.0 to 1.7.0

Upgraded the `rmcp` SDK dependency across a major version boundary (0.6.0 → 1.7.0).
The jump crosses the 1.0.0 release and several breaking changes; see
[Upstream rmcp changes](#upstream-rmcp-changes-060--170) below for the full upstream
history. The local code changes required to migrate were:

- **`Cargo.toml`**: bumped `rmcp` to `1.7.0` and **removed the `transport-sse-server`
  feature**. That feature was deleted upstream in rmcp 0.11.0
  ([#562](https://github.com/modelcontextprotocol/rust-sdk/pull/562), "remove SSE
  transport support"). It was already dead weight here — the `Sse` CLI subcommand is
  served by `StreamableHttpService` (the modern Streamable HTTP transport), not the
  legacy SSE server transport. No runtime behavior changes. The remaining features
  (`server`, `transport-io`, `transport-streamable-http-server`, `elicitation`,
  `schemars`) all still exist in 1.7.0.

- **`Parameters` import path** (`src/server.rs` + all 14 files under `src/tools/`):
  `rmcp::handler::server::tool::Parameters` is now private. The public re-export moved
  to `rmcp::handler::server::wrapper::Parameters`. Updated every import accordingly.

- **`ServerInfo` construction** (`src/server.rs`): `ServerInfo`
  (alias of `InitializeResult`) became `#[non_exhaustive]`, so it can no longer be built
  with a struct literal — even with `..Default::default()`. Rewrote `get_info()` to start
  from `ServerInfo::default()` and assign the `instructions` and `capabilities` fields.

- **Deprecation** (`src/server.rs`): the `initialize()` handler signature used
  `InitializeRequestParam`, now a deprecated alias. Switched to `InitializeRequestParams`.

- **Unused import** (`src/server.rs`): removed `use std::future::Future;`. It was only
  needed by the old `#[tool_router]`/`#[tool]` macro expansion; the current macros no
  longer require it in user scope.

Verified locally: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, and `cargo build --release` all pass. A stdio `initialize` + `tools/list`
smoke test negotiates protocol version `2025-06-18` and advertises all 55 tools.

---

## Upstream rmcp changes (0.6.0 → 1.7.0)

Curated from the [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
release notes. Items most relevant to this server are called out; the full per-version
list follows.

### Breaking changes that affected this project

- **0.11.0 — SSE transport removed** ([#562](https://github.com/modelcontextprotocol/rust-sdk/pull/562)).
  The `transport-sse-server` feature no longer exists. Handled by dropping the unused
  feature (see above).
- **1.x — `Parameters` re-export moved** to `handler::server::wrapper`; the old
  `handler::server::tool` path is private.
- **1.x — `InitializeResult`/`ServerInfo` is now `#[non_exhaustive]`**, forbidding
  struct-literal construction from downstream crates.
- **1.x — `InitializeRequestParam` deprecated** in favor of `InitializeRequestParams`.

### Other breaking / notable protocol changes (did not require local edits)

- **0.6.2 — batched JSON-RPC support removed** ([#408](https://github.com/modelcontextprotocol/rust-sdk/pull/408)).
- **0.6.3 — JSON-RPC request ID type changed** from `u32` to `i64`
  ([#416](https://github.com/modelcontextprotocol/rust-sdk/pull/416)).
- **0.9.1 — JSON Schema dialect default is now 2020-12**
  ([#549](https://github.com/modelcontextprotocol/rust-sdk/pull/549)).
- **0.11.0 — output-schema validation** for tools added
  ([#566](https://github.com/modelcontextprotocol/rust-sdk/pull/566)).
- **1.4.0 — macros can auto-generate `get_info` and a default router**
  ([#785](https://github.com/modelcontextprotocol/rust-sdk/pull/785)); this server still
  defines `get_info()` by hand.
- **1.5.0 — protocol version `2025-11-25` support added**
  ([#802](https://github.com/modelcontextprotocol/rust-sdk/pull/802)).
- **1.6.0 — Streamable HTTP server gained Origin/Host header validation**
  ([#823](https://github.com/modelcontextprotocol/rust-sdk/pull/823),
  [#827](https://github.com/modelcontextprotocol/rust-sdk/pull/827)), runtime tool
  disabling ([#809](https://github.com/modelcontextprotocol/rust-sdk/pull/809)), and an
  optional session store for resumability
  ([#775](https://github.com/modelcontextprotocol/rust-sdk/pull/775)).

### Themes across the range

- **OAuth / auth** saw heavy, ongoing work (discovery, DCR, CSRF, credential stores,
  client-credentials flow, OIDC). Not used by this server.
- **Streamable HTTP transport** matured significantly: JSON-or-SSE responses, custom
  headers, graceful shutdown, session re-init, Unix-socket client, stateless JSON mode.
- **MCP spec conformance**: numerous SEP implementations (tasks, elicitation schemas,
  sampling-with-tools, tool-name format, `_meta` fields, protocol-version header).

### Per-version reference

- **0.6.1** — auth header for streamable HTTP client; prompt support; resource_link in
  tools/prompts; graceful-shutdown fix.
- **0.6.2** — optional `_meta` on `CallToolResult`/`EmbeddedResource`/`ResourceContents`;
  **batched JSON-RPC removed**; LSP-notification compatibility fix.
- **0.6.3** — request ID `u32` → `i64`.
- **0.7.0** — OAuth fixes (CSRF requirement, public-client secrets, return auth errors).
- **0.8.0** — clients can override `client_name`; default schema generated for param-less
  tools; Rust 1.90.
- **0.8.1** — OAuth bearer-token propagation fix.
- **0.8.2** — type-safe elicitation schema support; `Icon.sizes` string → string array
  (SEP-973); OAuth discovery fixes.
- **0.8.3** — accept HTTP 204 (in addition to 202) on initialize.
- **0.8.4 / 0.8.5** — OAuth credential-refresh and protected-resource-discovery fixes.
- **0.9.0** — `CredentialStore` trait; `_meta` on tool definitions.
- **0.9.1** — Streamable HTTP supports both SSE and JSON responses; **JSON Schema 2020-12
  default dialect**; SEP-986 tool-name format.
- **0.10.0** — custom client notifications; `paste` → `pastey` in macros.
- **0.11.0** — **SSE transport removed (breaking)**; `_meta` on prompts/resources/paginated
  results; output-schema validation; streamable-HTTP graceful shutdown.
- **0.12.0** — custom requests and custom server notifications; SEP-991 (CIMD) URL-based
  client IDs; `cached_schema_for_type` merged into `schema_for_type`.
- **0.13.0** — blanket `ClientHandler`/`ServerHandler` impls; `close()` for graceful
  shutdown; `StateStore` trait; elicitation enum schema (SEP-1330); task support
  (SEP-1686); OIDC discovery.
- **0.14.0** — task capability modeling fixes; non-success HTTP no longer treated as a
  transport error; SEP-1319 (decouple request payload from RPC methods).
- **0.15.0** — URL elicitation (SEP-1036); native-tls backend option; capabilities
  `extensions` field (SEP-1724); sampling-with-tools (SEP-1577); various task fixes.
- **0.16.0** — custom HTTP headers in `StreamableHttpClient`; deterministic `list_all()`
  ordering; reqwest 0.13.2; 2025-11-25-compliant auth.
- **0.17.0** — stateless `json_response` server mode; trait-based tool declaration; default
  values for string/number/integer schemas; `MCP-Protocol-Version` header validation;
  allow empty content in `CallToolResult`.
- **1.0.0** — major release; API-ergonomics follow-ups; auth/streamable-HTTP fixes.
- **1.1.0 / 1.1.1** — OAuth 2.0 client-credentials flow; accept `logging/setLevel` and
  `ping` before the initialized notification.
- **1.2.0** — constructors for non-exhaustive model types; OAuth refresh-scope fix;
  ping-before-initialize handling; jsonwebtoken 9 → 10.
- **1.3.0** — Unix-socket streamable-HTTP client; transparent session re-init;
  `local` feature for `!Send` tool handlers; secret redaction in `Debug`; assorted
  `CallToolResult` deserialization fixes.
- **1.4.0** — macros auto-generate `get_info`/default router; `which_command` executable
  resolution; default session keep-alive 5 min; removed initialized-notification gate for
  Streamable HTTP.
- **1.5.0** — protocol version `2025-11-25` support; non-exhaustive transport error
  constructors; SSE connection-reuse fix.
- **1.6.0** — Origin/Host validation for Streamable HTTP; runtime tool disabling; optional
  session store (resumability); session `init_timeout`.
- **1.7.0** — task-based stdio examples; flatten `Resource` variant of
  `PromptMessageContent`; reply `-32700` on stdio parse errors instead of closing;
  dropped chrono default features.
