
#[test]
fn function_name_test() {
    println!("{:?}",std::fs::canonicalize("../ced/docs/meta.md").unwrap());
}
