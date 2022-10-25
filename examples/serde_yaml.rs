use serde::{Deserialize, Serialize};

fn good() -> String {
    String::from("good")
}

#[derive(Debug, Deserialize, Serialize)]
struct TestYaml {
    test: String,
    #[serde(skip_serializing_if = "String::is_empty", default = "good")]
    good: String,
}

impl TestYaml {
    fn from_str(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

fn main() {
    let a = TestYaml::from_str(
        r"--- 
test: xxxxx",
    );
    match a {
        Ok(r) => println!("{:?}", r),
        Err(e) => println!("{e}"),
    }
}
