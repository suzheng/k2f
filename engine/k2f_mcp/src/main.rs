use k2f_mcp::run_stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_stdio().await
}
