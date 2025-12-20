/// Basic demo test to verify test infrastructure works
#[test]
fn test_basic_addition() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_string_concatenation() {
    let greeting = format!("{} {}", "Hello", "World");
    assert_eq!(greeting, "Hello World");
}
