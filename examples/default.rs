fn main() {
    #[derive(Debug, Default)]
    struct Test {
        params: Vec<(String, String)>,
    }

    fn tst(a: &Test) {
        println!("{:?}", a);
    }

    tst(&Default::default());
}
