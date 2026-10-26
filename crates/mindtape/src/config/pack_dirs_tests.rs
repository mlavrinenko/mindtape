use super::absolute;

#[test]
fn absolute_resolves_dot_and_dot_dot_by_name() {
    let base = std::path::absolute(std::env::temp_dir()).unwrap_or_else(|_| std::env::temp_dir());
    let spelled = base.join("a").join(".").join("b").join("..").join("c");
    assert_eq!(absolute(&spelled), base.join("a").join("c"));
}
