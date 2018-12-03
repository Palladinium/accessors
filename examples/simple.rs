#[macro_use]
extern crate accessors;

#[derive(getters, setters)]
struct Simple {
    #[setter(into)]
    field_a: String,
    #[setter(into = true)]
    field_b: String,
}

fn main() {
    let mut s = Simple {
        field_a: "hello".to_owned(),
        field_b: "world".to_owned(),
    };

    println!("{}", s.field_a());
    s.set_field_a("hi");
    s.set_field_b("there");
}
