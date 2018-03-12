macro_rules! get_ref_block {
    (
        $in_file:expr, $no_meta:expr, $verbose:expr, $pr_verbosity_level:expr
    ) => {{
        let (ref_block_pos, ref_block) =
            match block_utils::get_ref_block($in_file,
                                             $no_meta,
                                             $pr_verbosity_level)? {
                None => { return Err(Error::with_message("Failed to find reference block")); },
                Some(x) => x,
            };

        if $verbose {
            println!();
            report_ref_block_info(ref_block_pos, &ref_block);
            println!();
        }

        (ref_block_pos, ref_block)
    }}
}

macro_rules! print_block {
    (
        $(
            $($arg:expr),*
        );*
    ) => {{
        $( println!($($arg),*); )*
    }}
}
