use reqwest::Client;
use serde_json::{Value, json};
use std::{
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::Duration,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

struct McpProcess(Child);

type HeaderCase<'a> = (&'a str, &'a [(&'a str, &'a str)], Option<&'a str>, bool);

impl Drop for McpProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn start_mcp(api_url: &str, fallback_key: Option<&str>) -> (McpProcess, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut command = Command::new(env!("CARGO_BIN_EXE_lago-mcp-server"));
    command
        .args(["sse", "--host", "127.0.0.1", "--port", &port.to_string()])
        .env("LAGO_API_URL", api_url)
        .env_remove("LAGO_API_KEY")
        .env_remove("LAGO_REGION")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(key) = fallback_key {
        command.env("LAGO_API_KEY", key);
    }
    let mut process = McpProcess(command.spawn().unwrap());
    let url = format!("http://127.0.0.1:{port}");
    let client = Client::new();
    for _ in 0..100 {
        assert!(
            process.0.try_wait().unwrap().is_none(),
            "MCP exited during startup"
        );
        if client.get(format!("{url}/health")).send().await.is_ok() {
            return (process, format!("{url}/mcp"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("MCP did not become ready");
}

async fn call_tool(url: &str, tool: &str, headers: &[(&str, &str)]) -> Value {
    let mut request = Client::new()
        .post(url)
        .timeout(Duration::from_secs(10))
        .header("Accept", "application/json, text/event-stream")
        .header("MCP-Protocol-Version", "2025-03-26")
        .json(&json!({"jsonrpc":"2.0", "id":1, "method":"tools/call",
            "params":{"name":tool,"arguments":{}}}));
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let body = request
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .text()
        .await
        .unwrap();
    let message: Value = serde_json::from_str(&body).unwrap_or_else(|_| {
        serde_json::from_str(
            body.lines()
                .find_map(|line| line.strip_prefix("data: "))
                .unwrap(),
        )
        .unwrap()
    });
    assert!(message.get("error").is_none(), "{message}");
    message["result"].clone()
}

#[tokio::test]
async fn both_headers_preserve_upstream_authentication_and_environment_fallback() {
    let api = MockServer::start().await;
    let api_url = format!("{}/api/v1", api.uri());
    let (_process, url) = start_mcp(&api_url, None).await;
    let (_fallback_process, fallback_url) = start_mcp(&api_url, Some("test-env-key")).await;

    // Exercise both shared configuration paths: SDK-backed customers and direct HTTP events.
    for (tool, resource) in [("list_customers", "customers"), ("list_events", "events")] {
        let cases: &[HeaderCase<'_>] = &[
            (
                &url,
                &[("x-lago-api-key", "test-legacy-key")],
                Some("test-legacy-key"),
                true,
            ),
            (
                &url,
                &[("x-api-key", "test-standard-key")],
                Some("test-standard-key"),
                true,
            ),
            (
                &url,
                &[("X-API-KEY", "test-standard-key")],
                Some("test-standard-key"),
                true,
            ),
            (
                &url,
                &[
                    ("x-lago-api-key", "test-legacy-key"),
                    ("x-api-key", "test-other-key"),
                ],
                Some("test-legacy-key"),
                true,
            ),
            (&url, &[], None, false),
            (
                &url,
                &[("x-api-key", "invalid-key")],
                Some("invalid-key"),
                false,
            ),
            (
                &url,
                &[("x-lago-api-key", "invalid-key")],
                Some("invalid-key"),
                false,
            ),
            (
                &url,
                &[
                    ("x-lago-api-key", "invalid-key"),
                    ("x-api-key", "test-standard-key"),
                ],
                Some("invalid-key"),
                false,
            ),
            (&fallback_url, &[], Some("test-env-key"), true),
            (
                &fallback_url,
                &[("x-api-key", "test-standard-key")],
                Some("test-standard-key"),
                true,
            ),
            (
                &fallback_url,
                &[("x-api-key", "invalid-key")],
                Some("invalid-key"),
                false,
            ),
        ];
        for (index, (endpoint, headers, expected_key, success)) in cases.iter().enumerate() {
            api.reset().await;
            if let Some(key) = expected_key {
                let response = if *success {
                    ResponseTemplate::new(200).set_body_json(json!({
                        resource: [], "meta": {"current_page":1,"total_pages":0,"total_count":0,"next_page":null,"prev_page":null}
                    }))
                } else {
                    ResponseTemplate::new(401)
                        .set_body_json(json!({"status":401,"error":"Unauthorized"}))
                };
                Mock::given(method("GET"))
                    .and(path(format!("/api/v1/{resource}")))
                    .and(header("Authorization", format!("Bearer {key}")))
                    .respond_with(response)
                    .expect(1)
                    .mount(&api)
                    .await;
            }
            let result = call_tool(endpoint, tool, headers).await;
            assert_eq!(
                result["isError"].as_bool().unwrap_or(false),
                !success,
                "{tool} case {index}: {result}"
            );
            let requests = api.received_requests().await.unwrap();
            assert_eq!(
                requests.len(),
                usize::from(expected_key.is_some()),
                "{tool} case {index}"
            );
            api.verify().await;
        }
    }
}
