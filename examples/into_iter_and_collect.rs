use std::collections::HashMap;

fn main() {
    let arr = vec![("profile", "todo")];
    let bb = arr.into_iter().collect::<HashMap<&str, &str>>();
    println!("{:?}", bb);
}
