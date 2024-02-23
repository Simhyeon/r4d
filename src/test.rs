use crate::common::MacroAttribute;
use crate::utils::{RadStr, Utils};
use crate::ArgParser;

#[test]
fn test() {
    let ret = "a b c d e f g
1 2 3 4 5 6 7 "
        .full_lines_with_index()
        .collect::<Vec<_>>();
    eprintln!("{:#?}", ret);
}
