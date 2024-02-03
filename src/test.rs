use crate::utils::Utils;
use crate::NewArgParser;

#[test]
fn arg_test() {
    let test = "a\nb\nc";
    let matched = test.match_indices('\n').collect::<Vec<_>>();
    let matched = Utils::full_lines(test).collect::<Vec<_>>();
}
