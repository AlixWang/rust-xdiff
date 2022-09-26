use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();

    map.insert("hello", "world");

    let v = map.get("hello").map(|v| v.as_bytes());

    let str = String::from("1111;22222");

    let mut str1 = str.split(';');

    println!("{:?},{:?}", str1.next(), str1.next());
    println!("{:?}", v);
}
