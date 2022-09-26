use anyhow::Result;
use reqwest::{Client, Method};

#[tokio::main]
async fn main() -> Result<()> {
    let req = Client::new();
    let mut res = req
        .request(Method::GET, "https://www.google.com")
        .send()
        .await?;

    while let Some(chunk) = res.chunk().await? {
        print!("{:?}", chunk);
    }
    Ok(())
}
