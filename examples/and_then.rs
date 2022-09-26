fn main() {
    let food = Some("food");
    let food1 = food.and_then(|f| Some(f.as_bytes()));
    println!("{:?}", food1);
}
