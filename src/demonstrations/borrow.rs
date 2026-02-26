fn borrow() {
    let s = String::from("Hello DoC");
    let x = 67;

    print_string(&s);
    print_integer(x);

    println!("what is s? {s}");
    println!("what is x? {x}");
}

fn print_string(string: &String) {
    println!("{string}");
}

fn print_integer(integer: i32) {
    println!("{integer}");
}
