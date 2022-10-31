#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let o: serde_json::Value = serde_json::from_slice(b"{'a':'1'}")?;
    println!("{}", o);
    Ok(())
}
