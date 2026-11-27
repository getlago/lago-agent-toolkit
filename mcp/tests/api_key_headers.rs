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
    start_mcp_with_hosts(api_url, fallback_key, None).await
}

async fn start_mcp_with_hosts(
    api_url: &str,
    fallback_key: Option<&str>,
    allowed_hosts: Option<&str>,
) -> (McpProcess, String) {
    let mut env = vec![("LAGO_API_URL", api_url)];
    env.extend(fallback_key.map(|key| ("LAGO_API_KEY", key)));
    env.extend(allowed_hosts.map(|hosts| ("MCP_ALLOWED_HOSTS", hosts)));
    spawn_mcp(&env).await
}

async fn spawn_mcp(env: &[(&str, &str)]) -> (McpProcess, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut command = Command::new(env!("CARGO_BIN_EXE_lago-mcp-server"));
    command
        .args(["sse", "--host", "127.0.0.1", "--port", &port.to_string()])
        .env_remove("LAGO_API_URL")
        .env_remove("LAGO_API_KEY")
        .env_remove("LAGO_REGION")
        .env_remove("MCP_ALLOWED_HOSTS")
        .envs(env.iter().copied())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
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

async fn post_tool(url: &str, tool: &str, headers: &[(&str, &str)]) -> reqwest::Response {
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
    request.send().await.unwrap()
}

async fn call_tool(url: &str, tool: &str, headers: &[(&str, &str)]) -> Value {
    let body = post_tool(url, tool, headers)
        .await
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
async fn api_key_headers_are_forwarded_upstream_with_precedence() {
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
            (
                &url,
                &[("authorization", "Bearer test-bearer-key")],
                Some("test-bearer-key"),
                true,
            ),
            (
                &url,
                &[("Authorization", "bearer test-bearer-key")],
                Some("test-bearer-key"),
                true,
            ),
            (
                &url,
                &[
                    ("x-api-key", "test-standard-key"),
                    ("authorization", "Bearer test-bearer-key"),
                ],
                Some("test-standard-key"),
                true,
            ),
            (
                &url,
                &[
                    ("x-lago-api-key", "test-legacy-key"),
                    ("x-api-key", "test-standard-key"),
                    ("authorization", "Bearer test-bearer-key"),
                ],
                Some("test-legacy-key"),
                true,
            ),
            (
                &url,
                &[
                    ("x-api-key", ""),
                    ("authorization", "Bearer test-bearer-key"),
                ],
                Some("test-bearer-key"),
                true,
            ),
            (
                &url,
                &[("authorization", "Bearer invalid-key")],
                Some("invalid-key"),
                false,
            ),
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
            (
                &fallback_url,
                &[("authorization", "Bearer test-bearer-key")],
                Some("test-bearer-key"),
                true,
            ),
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

#[tokio::test]
async fn http_requests_without_an_api_key_are_rejected_with_401() {
    let api = MockServer::start().await;
    let api_url = format!("{}/api/v1", api.uri());
    let (_process, url) = start_mcp(&api_url, None).await;
    // The env key must never be used in HTTP mode, even when it is set.
    let (_fallback_process, fallback_url) = start_mcp(&api_url, Some("test-env-key")).await;

    let cases: &[&[(&str, &str)]] = &[
        &[],
        &[("x-api-key", "")],
        &[("x-lago-api-key", "  ")],
        &[("authorization", "Bearer ")],
        &[("authorization", "Basic dXNlcjpwYXNz")],
        &[("authorization", "test-bare-key")],
    ];
    for endpoint in [&url, &fallback_url] {
        for (index, headers) in cases.iter().enumerate() {
            let response = post_tool(endpoint, "list_customers", headers).await;
            assert_eq!(response.status(), 401, "case {index} on {endpoint}");
            assert_eq!(
                response.headers()["www-authenticate"],
                "Bearer",
                "case {index} on {endpoint}"
            );
        }
        let list = Client::new()
            .post(endpoint.as_str())
            .header("Accept", "application/json, text/event-stream")
            .json(&json!({"jsonrpc":"2.0", "id":1, "method":"tools/list"}))
            .send()
            .await
            .unwrap();
        assert_eq!(list.status(), 401, "tools/list on {endpoint}");
        let health = Client::new()
            .get(endpoint.replace("/mcp", "/health"))
            .send()
            .await
            .unwrap();
        assert_eq!(health.status(), 200);
    }
    assert!(api.received_requests().await.unwrap().is_empty());
}

async fn mount_customers_ok(api: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/customers"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "customers": [], "meta": {"current_page":1,"total_pages":0,"total_count":0,"next_page":null,"prev_page":null}
        })))
        .mount(api)
        .await;
}

#[tokio::test]
async fn host_and_origin_are_validated_when_allowed_hosts_is_set() {
    let api = MockServer::start().await;
    mount_customers_ok(&api).await;
    let api_url = format!("{}/api/v1", api.uri());
    let (_process, url) =
        start_mcp_with_hosts(&api_url, None, Some(" mcp.example.com , other.example.com")).await;

    let key = ("x-api-key", "test-key");
    let allowed: &[&[(&str, &str)]] = &[
        &[key, ("host", "mcp.example.com")],
        &[key, ("host", "other.example.com")],
        &[
            key,
            ("host", "mcp.example.com"),
            ("origin", "https://mcp.example.com"),
        ],
    ];
    for (index, headers) in allowed.iter().enumerate() {
        let response = post_tool(&url, "list_customers", headers).await;
        assert_eq!(response.status(), 200, "allowed case {index}");
    }

    let rejected: &[&[(&str, &str)]] = &[
        // Default Host sent by the client is 127.0.0.1:<port>, which is not listed.
        &[key],
        &[key, ("host", "evil.example.com")],
        &[
            key,
            ("host", "mcp.example.com"),
            ("origin", "https://evil.example.com"),
        ],
        &[
            key,
            ("host", "mcp.example.com"),
            ("origin", "http://mcp.example.com"),
        ],
    ];
    for (index, headers) in rejected.iter().enumerate() {
        let response = post_tool(&url, "list_customers", headers).await;
        assert_eq!(response.status(), 403, "rejected case {index}");
    }
}

#[tokio::test]
async fn host_and_origin_are_not_validated_when_allowed_hosts_is_unset() {
    let api = MockServer::start().await;
    mount_customers_ok(&api).await;
    let api_url = format!("{}/api/v1", api.uri());
    let (_process, url) = start_mcp(&api_url, None).await;

    let key = ("x-lago-api-key", "test-key");
    let cases: &[&[(&str, &str)]] = &[
        &[key],
        &[key, ("host", "evil.example.com")],
        &[key, ("origin", "https://evil.example.com")],
    ];
    for (index, headers) in cases.iter().enumerate() {
        let result = call_tool(&url, "list_customers", headers).await;
        assert!(
            !result["isError"].as_bool().unwrap_or(false),
            "case {index}: {result}"
        );
    }
}

#[tokio::test]
async fn header_key_is_used_when_lago_api_url_is_unset() {
    let api = MockServer::start().await;
    let api_url = format!("{}/api/v1", api.uri());
    // Region resolved from LAGO_REGION, with a server-side key that must not be used.
    let (_process, url) =
        spawn_mcp(&[("LAGO_REGION", &api_url), ("LAGO_API_KEY", "test-env-key")]).await;

    for (tool, resource) in [("list_customers", "customers"), ("list_events", "events")] {
        api.reset().await;
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/{resource}")))
            .and(header("Authorization", "Bearer test-standard-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                resource: [], "meta": {"current_page":1,"total_pages":0,"total_count":0,"next_page":null,"prev_page":null}
            })))
            .expect(1)
            .mount(&api)
            .await;
        let result = call_tool(&url, tool, &[("x-api-key", "test-standard-key")]).await;
        assert!(
            !result["isError"].as_bool().unwrap_or(false),
            "{tool}: {result}"
        );
        api.verify().await;
    }
}
