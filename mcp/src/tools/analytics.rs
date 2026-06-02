// APPROVED SECURITY EXCEPTION (see CLAUDE.md):
//
// Every other tool in this crate reaches data through `lago-client` -> Lago Rails,
// and the `X-LAGO-API-KEY` is only ever forwarded to Rails. This tool is an
// intentional, reviewed exception: it calls the first-party Lago analytics agent
// (`agent.getlago.com`) directly over HTTP via `reqwest`, NOT through `lago-client`.
//
// Why this is still safe and org-isolated:
//   - The agent is a first-party Lago service, not a third party. The api_key we
//     forward is re-validated by the agent against the Lago REST API on every call,
//     so an invalid/forged key is rejected upstream exactly as Rails would reject it.
//   - After resolving the org from that key, the agent applies row-level security
//     keyed off the resolved org, so a caller can only ever see their own org's rows.
//   - The agent queries a read-only analytical Postgres replica purpose-built for
//     text-to-SQL reporting, not arbitrary application storage. There is no write
//     path and no cross-org data surface.
//
// Net effect: the same key boundary and org isolation guarantees hold; only the
// transport (direct reqwest to a first-party analytics service) differs from the
// rest of the crate.

// the service is constructed and called from server.rs once the tool is registered
// (next task); until then these items are unreferenced outside the test module.
#![allow(dead_code)]

use rmcp::{
    RoleServer,
    handler::server::tool::Parameters,
    model::{CallToolResult, Content},
    service::RequestContext,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use crate::tools::{error_result, get_lago_api_config};

// default agent request timeout when LAGO_AGENT_TIMEOUT_SECS is unset/invalid
const DEFAULT_AGENT_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AskParams {
    /// Natural-language question about your Lago billing/usage data. The analytics agent
    /// turns it into SQL, runs it, and returns the SQL, an explanation, and a results table.
    pub question: String,
    /// Optional session_id returned by a previous ask_lago_analytics response. Pass it back
    /// to refine the previous result (filter, aggregate, sort) using the agent's cached rows.
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentAskRequest {
    question: String,
    // omit the field entirely on a fresh ask so the agent treats it as a new session
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentAskResponse {
    sql_query: String,
    explanation: String,
    results: String,
    session_id: String,
    // the agent only sets this when a passed-in session_id had expired and it answered fresh
    #[serde(default)]
    session_expired: bool,
}

// pure renderer: no I/O, no RequestContext, so it is unit-testable in isolation.
fn render_markdown(resp: &AgentAskResponse) -> String {
    // surface to the caller that their follow-up session was gone and this is a fresh answer
    let expired_note = if resp.session_expired {
        "(previous session expired; this was answered fresh)\n\n"
    } else {
        ""
    };

    format!(
        "```sql\n{sql}\n```\n\n{expired}{explanation}\n\n{results}\n\n---\n\nFollow-up: call this tool again with the `session_id` below to refine this result via the agent's cached rows.\n\n```json\n{{\"session_id\": \"{session_id}\"}}\n```\n",
        sql = resp.sql_query,
        expired = expired_note,
        explanation = resp.explanation,
        results = resp.results,
        session_id = resp.session_id,
    )
}

// takes the resolved api_key + agent_url as plain params (no RequestContext), so the
// HTTP path is fully testable with a mock server. returns the parsed response or an
// error CallToolResult ready to hand back to the caller.
async fn call_agent(
    client: &reqwest::Client,
    agent_url: &str,
    api_key: &str,
    body: &AgentAskRequest,
) -> Result<AgentAskResponse, CallToolResult> {
    let ask_url = format!("{}/ask", agent_url.trim_end_matches('/'));

    let response = client
        .post(&ask_url)
        .header("X-LAGO-API-KEY", api_key)
        .json(body)
        .send()
        .await
        .map_err(|e| {
            // distinguish a timeout from a connection/transport failure for a clearer message
            if e.is_timeout() {
                error_result("Analytics agent request timed out")
            } else {
                error_result(format!("Analytics agent unreachable: {e}"))
            }
        })?;

    let status = response.status();
    if !status.is_success() {
        // forward the upstream status + body so the caller can see the agent's own error
        let body_text = response.text().await.unwrap_or_default();
        return Err(error_result(format!(
            "Analytics agent returned {status}: {body_text}"
        )));
    }

    response
        .json::<AgentAskResponse>()
        .await
        .map_err(|e| error_result(format!("Failed to parse agent response: {e}")))
}

#[derive(Clone)]
pub struct AnalyticsService;

impl AnalyticsService {
    pub fn new() -> Self {
        Self
    }

    pub async fn ask(
        &self,
        Parameters(params): Parameters<AskParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // reuse the crate's dual-mode key resolution; we only need the api_key here,
        // not its base_url (that is the Lago REST url, not the agent url).
        let api_key = match get_lago_api_config(&context).await {
            Ok(config) => config.api_key,
            Err(error_result) => return Ok(error_result),
        };

        // the agent url is deployment config on the MCP server, never a request header
        let agent_url = env::var("LAGO_AGENT_API_URL").unwrap_or_default();
        if agent_url.is_empty() {
            return Ok(error_result(
                "Analytics agent not configured: set LAGO_AGENT_API_URL on the MCP server.",
            ));
        }

        // operators can tune the timeout; fall back to a sane default on unset/garbage input
        let timeout_secs = env::var("LAGO_AGENT_TIMEOUT_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(DEFAULT_AGENT_TIMEOUT_SECS);

        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
        {
            Ok(client) => client,
            Err(e) => return Ok(error_result(format!("Failed to build HTTP client: {e}"))),
        };

        let request_body = AgentAskRequest {
            question: params.question,
            session_id: params.session_id,
        };

        match call_agent(&client, &agent_url, &api_key, &request_body).await {
            Err(error_result) => Ok(error_result),
            // happy path returns Markdown, not serialized JSON, so build the result directly
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                render_markdown(&resp),
            )])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_response(session_expired: bool) -> AgentAskResponse {
        AgentAskResponse {
            sql_query: "SELECT count(*) FROM invoices".to_string(),
            explanation: "Counts all invoices in your org.".to_string(),
            results: "| count |\n| --- |\n| 42 |".to_string(),
            session_id: "sess-abc-123".to_string(),
            session_expired,
        }
    }

    #[test]
    fn render_markdown_includes_sql_explanation_results_and_session_json() {
        let resp = sample_response(false);
        let out = render_markdown(&resp);

        assert!(out.contains("SELECT count(*) FROM invoices"));
        assert!(out.contains("Counts all invoices in your org."));
        assert!(out.contains("| count |"));
        assert!(out.contains("```json"));
        assert!(out.contains("\"session_id\": \"sess-abc-123\""));
    }

    #[test]
    fn render_markdown_prepends_expired_note_when_session_expired() {
        let resp = sample_response(true);
        let out = render_markdown(&resp);

        assert!(out.contains("previous session expired"));
    }

    #[test]
    fn render_markdown_omits_expired_note_when_not_expired() {
        let resp = sample_response(false);
        let out = render_markdown(&resp);

        assert!(!out.contains("previous session expired"));
    }

    #[tokio::test]
    async fn call_agent_returns_parsed_response_on_200() {
        let server = MockServer::start().await;

        let response_body = serde_json::json!({
            "sql_query": "SELECT 1",
            "explanation": "ok",
            "results": "| 1 |",
            "session_id": "sess-xyz",
            "session_expired": false,
        });

        // matching on the header proves we forward the key (Finding #1 regression guard)
        Mock::given(method("POST"))
            .and(path("/ask"))
            .and(header("X-LAGO-API-KEY", "test-key-42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .expect(1)
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let body = AgentAskRequest {
            question: "how many?".to_string(),
            session_id: None,
        };

        let result = call_agent(&client, &server.uri(), "test-key-42", &body).await;

        let parsed = result.expect("expected Ok response from agent");
        assert_eq!(parsed.sql_query, "SELECT 1");
        assert_eq!(parsed.explanation, "ok");
        assert_eq!(parsed.results, "| 1 |");
        assert_eq!(parsed.session_id, "sess-xyz");
        assert!(!parsed.session_expired);
    }

    #[tokio::test]
    async fn call_agent_returns_error_result_on_non_2xx() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ask"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let client = reqwest::Client::new();
        let body = AgentAskRequest {
            question: "how many?".to_string(),
            session_id: None,
        };

        let result = call_agent(&client, &server.uri(), "bad-key", &body).await;

        let error = result.expect_err("expected Err(CallToolResult) on non-2xx");
        assert_eq!(error.is_error, Some(true));
    }
}
