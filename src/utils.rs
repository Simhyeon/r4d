pub(crate) fn args_to_vec<'a>(args : &'a str) -> Vec<&'a str> {
    args.split(",").collect()
}

pub(crate) fn args_with_len<'a>(args: &'a str, length: usize) -> Option<Vec<&'a str>> {
    let args: Vec<_> = args.split(",").collect();

    if args.len() != length {
        return None;
    } 

    Some(args)
}
