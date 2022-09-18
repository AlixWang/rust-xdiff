use anyhow::Result;
use xdiff::DiffConfig;

async fn load_yaml(path: &str) -> Result<DiffConfig> {

  let res = xdiff::DiffConfig::load_yaml(path).await?;
  Ok(res)
}

#[tokio::main]
async fn main() -> Result<()> {
  let res = load_yaml("/fixtures/test.yaml").await;
  match res {
    Ok(c) => println!("{:?}", c),
    Err(e) => panic!("{}",e),
}
  Ok(())
}