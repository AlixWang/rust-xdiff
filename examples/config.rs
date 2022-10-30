use anyhow::Result;
use xdiff::{DiffConfig, ExtraArgs};

async fn load_yaml(path: &str) -> Result<DiffConfig> {
    let res = xdiff::DiffConfig::load_yaml(path).await?;
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<()> {
    let res = load_yaml("./fixtures/test.yaml").await;
    let c = match res {
        Ok(c) => c,
        Err(_e) => todo!(),
    };
    let b = c
        .get_profile("todo")
        .unwrap()
        .diff(ExtraArgs {
            headers: vec![],
            body: vec![],
            query: vec![],
        })
        .await?;
    Ok(())
}
