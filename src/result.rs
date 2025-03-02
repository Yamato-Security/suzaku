pub fn s(input: String) -> String {
    input.replace(r#"Value(String(""#, "").replace(r#""))"#, "")
}
