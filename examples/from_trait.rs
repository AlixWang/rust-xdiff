#[derive(Debug)]
struct Test {
    name: String,
}

impl From<String> for Test {
    fn from(test: String) -> Self {
        Self { name: test }
    }
}

fn main() {
    let test = Test::from(String::from("xxx"));
    println!("{:?}",test);
}
