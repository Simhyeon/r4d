use crate::utils::Utils;
use crate::NewArgParser;

#[test]
fn arg_test() {
    eprintln!(
        "{:#?}",
        Utils::get_whitespace_split_retain_quote_rule("a b ' c f g ' d")
    );
    let mut arg_parser = NewArgParser::new();
    let result = arg_parser.args_to_vec("\\(,a", ',', crate::SplitVariant::Always);
    eprintln!("{result:#?}");
}
