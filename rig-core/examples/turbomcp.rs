//! An example of how you can use `turbomcp` with Rig to create an MCP friendly agent.
//!
//! This example demonstrates a complete end-to-end flow:
//! 1. Start a TurboMCP server with math tools
//! 2. Connect a client to the server
//! 3. Create a Rig agent with the tools
//! 4. Use the agent to perform calculations via tool calls
//!

#![allow(unexpected_cfgs)]

#[cfg(feature = "turbomcp")]
use turbomcp::prelude::*;

#[cfg(feature = "turbomcp")]
use rig::{client::CompletionClient, completion::Prompt, providers::openai};

#[cfg(feature = "turbomcp")]
use turbomcp_client::{StreamableHttpClientConfig, StreamableHttpClientTransport};

#[cfg(feature = "turbomcp")]
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MathRequest {
    pub a: i32,
    pub b: i32,
}

#[cfg(feature = "turbomcp")]
#[derive(Clone)]
pub struct MathServer;

#[cfg(feature = "turbomcp")]
#[turbomcp::server(name = "turbomcp-rig-demo", version = "1.0.0", transports = ["http"])]
impl MathServer {
    /// Calculate the sum of two numbers
    #[tool("Calculate the sum of two numbers")]
    async fn sum(&self, request: MathRequest) -> McpResult<String> {
        Ok((request.a + request.b).to_string())
    }
}

#[cfg(feature = "turbomcp")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Spawn the MCP server on HTTP
    tokio::spawn(async move {
        if let Err(e) = MathServer
            .run_http_with_path("127.0.0.1:8080", "/mcp")
            .await
        {
            tracing::error!("Server error: {e}");
        }
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Connect client to the server
    let transport = StreamableHttpClientTransport::new(StreamableHttpClientConfig {
        base_url: "http://localhost:8080".to_string(),
        endpoint_path: "/mcp".to_string(),
        ..Default::default()
    });
    let client = turbomcp_client::Client::new(transport);

    // Initialize
    let server_info = client.initialize().await?;
    tracing::info!("Connected to server: {:#?}", server_info.server_info);

    // List tools
    let tools = client.list_tools().await?;

    // takes the `OPENAI_API_KEY` as an env var on usage
    let openai_client = openai::Client::from_env();
    let agent = openai_client
        .agent("gpt-4o")
        .preamble("You are a helpful assistant who has access to a math tool server.")
        .turbomcp_tools(tools, client)
        .build();

    let res = agent.prompt("What is 2+5?").multi_turn(2).await?;

    println!("GPT-4o: {res}");

    Ok(())
}

#[cfg(not(feature = "turbomcp"))]
fn main() {
    eprintln!(
        "This example requires the 'turbomcp' feature. Run with: cargo run --example turbomcp --features turbomcp,derive"
    );
}
