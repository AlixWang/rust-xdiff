struct Test {
    name: String,
    age: i32,
}

fn main() {
    let o = Some("Hello world".to_owned());

    match o.as_ref() {
        Some(x) => println!("{:?}", x),
        _ => println!("hello"),
    };

    println!("{:?}", o);

    let b = Some(Test {
        name: String::new(),
        age: 12,
    });
}

fn test_ref(s: &str) {
    println!("{:?}", s);
}
