macro_rules! unwrap_or {
    (
        $val:expr, $or:expr
    ) => {{
        match $val {
            Some(x) => x,
            None    => $or
        }
    }}
}

macro_rules! get_ref_block {
    (
        $param:expr, $no_meta:expr, $stop_flag:expr
    ) => {{
        use std::sync::atomic::Ordering;

        let (ref_block_pos, ref_block) =
            match block_utils::get_ref_block(&$param.in_file,
                                             $no_meta,
                                             $param.pr_verbosity_level,
                                             &$stop_flag)? {
                None => {
                    if $stop_flag.load(Ordering::SeqCst) {
                        return Ok(None)
                    } else {
                        return Err(Error::with_message("Failed to find reference block"));
                    }
                },
                Some(x) => x,
            };

        if $param.verbose {
            println!();
            report_ref_block_info(ref_block_pos, &ref_block);
            println!();
        }

        (ref_block_pos, ref_block)
    }};
    (
        $param:expr, $stop_flag:expr
    ) => {{
        get_ref_block!($param, $param.no_meta, $stop_flag)
    }}
}

macro_rules! print_block {
    (
        $(
            $($arg:expr),*;
        )*
    ) => {{
        $( println!($($arg),*) );*
    }}
}

macro_rules! get_RSD_from_ref_block {
    (
        $ref_block_pos:expr, $ref_block:expr, $purpose:expr
    ) => {{
        let ver_usize = ver_to_usize($ref_block.get_version());
        match $ref_block.get_RSD().unwrap() {
            None    => {
                return Err(Error::with_message(&format!("Reference block at byte {} (0x{:X}) is a metadata block but does not have RSD field(must be present to {} for version {})",
                                                        $ref_block_pos,
                                                        $ref_block_pos,
                                                        $purpose,
                                                        ver_usize)));
            },
            Some(x) => x as usize,
        }
    }}
}

macro_rules! get_RSP_from_ref_block {
    (
        $ref_block_pos:expr, $ref_block:expr, $purpose:expr
    ) => {{
        let ver_usize = ver_to_usize($ref_block.get_version());
        match $ref_block.get_RSP().unwrap() {
            None    => {
                return Err(Error::with_message(&format!("Reference block at byte {} (0x{:X}) is a metadata block but does not have RSP field({} for version {})",
                                                        $ref_block_pos,
                                                        $ref_block_pos,
                                                        $purpose,
                                                        ver_usize)));
            },
            Some(x) => x as usize,
        }
    }}
}

macro_rules! return_if_not_ver_uses_rs {
    (
        $version:expr
    ) => {{
        use sbx_specs::*;
        if !ver_uses_rs($version) {
            println!("Version {} does not use Reed-Solomon erasure code, exiting now", ver_to_usize($version));
            return Ok(None);
        }
    }}
}

macro_rules! return_if_ref_not_meta {
    (
        $ref_block_pos:expr, $ref_block:expr, $purpose:expr
    ) => {{
        if $ref_block.is_data() {
            let ver_usize = ver_to_usize($ref_block.get_version());
            return Err(Error::with_message(&format!("Reference block at byte {} (0x{:X}) is not a metadata block(metadata block must be used to {} for version {})",
                                                    $ref_block_pos,
                                                    $ref_block_pos,
                                                    $purpose,
                                                    ver_usize)));
        }
    }}
}

macro_rules! get_burst_or_guess {
    (
        $param:expr, $ref_block_pos:expr, $ref_block:expr
    ) => {{
        let burst = unwrap_or!($param.burst,
                               if ver_uses_rs($ref_block.get_version()) {
                                   unwrap_or!(block_utils::guess_burst_err_resistance_level(&$param.in_file,
                                                                                            $ref_block_pos,
                                                                                            &$ref_block)?,
                                              {
                                                  return Err(
                                                      Error::with_message(
                                                          "Failed to guess burst resistance level, please specify via --burst option"));
                                              })
                               } else {
                                   0
                               });

        print_if_verbose!($param =>
                          "Using burst error resistance level {} for output container", burst;
                          "";
        );

        burst
    }}
}

macro_rules! print_if_verbose {
    (
        $param:expr =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $param.verbose {
            print_block!(
                $( $($expr),*; )*
            );
        }
    }};
    (
        $param:expr, $reporter:expr =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $param.verbose {
            pause_reporter!($reporter =>
                            print_block!(
                                $( $($expr),*; )*
                            );
            );
        }
    }}
}

macro_rules! pause_reporter {
    (
        $reporter:expr =>
            $($expr:expr;)*
    ) => {{
        $reporter.pause();
        $($expr;)*;
        $reporter.resume();
    }}
}

macro_rules! break_if_atomic_bool {
    (
        $atomic_bool:expr
    ) => {{
        use std::sync::atomic::Ordering;
        if $atomic_bool.load(Ordering::SeqCst) {
            break;
        }
    }}
}
