macro_rules! unwrap_or {
    (
        $val:expr, $or:expr
    ) => {{
        match $val {
            Some(x) => x,
            None => $or,
        }
    }};
}

macro_rules! get_ref_block {
    (
        $param:expr, $json_printer:expr, $stop_flag:expr
    ) => {{
        get_ref_block!($param, $json_printer, $param.ref_block_choice, $stop_flag)
    }};
    (
        $param:expr, $json_printer:expr, $ref_block_choice:expr, $stop_flag:expr
    ) => {{
        use crate::sbx_specs::SBX_SCAN_BLOCK_SIZE;

        let from_pos = match $param.ref_block_from_pos {
            Some(x) => Some(x),
            None => {
                match $param.from_pos {
                    Some(x) => Some(x % SBX_SCAN_BLOCK_SIZE as u64),
                    None => None
                }
            },
        };

        get_ref_block!($param,
                       from_pos,
                       $param.ref_block_to_pos,
                       $json_printer,
                       $ref_block_choice,
                       $stop_flag)
    }};
    (
        no_force_misalign => $param:expr, $json_printer:expr, $ref_block_choice:expr, $stop_flag:expr
    ) => {{
        get_ref_block!(no_force_misalign =>
                       $param,
                       $param.ref_block_from_pos,
                       $param.ref_block_to_pos,
                       $json_printer,
                       $ref_block_choice,
                       $stop_flag)
    }};
    (
        $param:expr, $ref_block_from_pos:expr, $ref_block_to_pos:expr, $json_printer:expr, $ref_block_choice:expr, $stop_flag:expr
    ) => {{
        get_ref_block!($param,
                       $ref_block_from_pos,
                       $ref_block_to_pos,
                       $param.force_misalign,
                       $json_printer,
                       $ref_block_choice,
                       $stop_flag)
    }};
    (
        no_force_misalign => $param:expr, $ref_block_from_pos:expr, $ref_block_to_pos:expr, $json_printer:expr, $ref_block_choice:expr, $stop_flag:expr
    ) => {{
        get_ref_block!($param,
                       $ref_block_from_pos,
                       $ref_block_to_pos,
                       false,
                       $json_printer,
                       $ref_block_choice,
                       $stop_flag)
    }};
    (
        $param:expr, $ref_block_from_pos:expr, $ref_block_to_pos:expr, $force_misalign:expr, $json_printer:expr, $ref_block_choice:expr, $stop_flag:expr
    ) => {{
        use std::sync::atomic::Ordering;
        use crate::cli_utils::report_ref_block_info;
        use crate::block_utils;

        let (ref_block_pos, ref_block) =
            match block_utils::get_ref_block(&$param.in_file,
                                             $ref_block_from_pos,
                                             $ref_block_to_pos,
                                             $force_misalign,
                                             $ref_block_choice,
                                             $param.pr_verbosity_level,
                                             $param.json_printer.json_enabled(),
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
            print_if!(not_json => $json_printer => "";);
            report_ref_block_info($json_printer, ref_block_pos, &ref_block);
            print_if!(not_json => $json_printer => "";);
        }

        (ref_block_pos, ref_block)
    }};
}

macro_rules! print_block {
    (
        $(
            $($arg:expr),*;
        )*
    ) => {{
        $( println!($($arg),*) );*
    }};
    (
        $output_channel:expr =>
        $(
            $($arg:expr),*;
        )*
    ) => {{
        $( println_at_output_channel!($output_channel => $($arg),*) );*
    }}
}

macro_rules! write_block {
    (
        $f:expr,
        $(
            $($arg:expr),*;
        )*
    ) => {{
        $( writeln!($f, $($arg),*)? );*
    }}
}

macro_rules! get_RSD_from_ref_block {
    (
        $ref_block_pos:expr, $ref_block:expr, $purpose:expr
    ) => {{
        use crate::sbx_specs::ver_to_usize;
        use crate::general_error::Error;

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
        use crate::sbx_specs::ver_to_usize;

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
        $version:expr, $json_printer:expr
    ) => {{
        use crate::sbx_specs::*;
        if !ver_uses_rs($version) {
            print_if!(not_json => $json_printer => "Version {} does not use Reed-Solomon erasure code, exiting now", ver_to_usize($version););
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
            return Err(Error::with_message(&format!("Reference block at byte {} (0x{:X}) is not a metadata block (metadata block must be used to {} for version {})",
                                                    $ref_block_pos,
                                                    $ref_block_pos,
                                                    $purpose,
                                                    ver_usize)));
        }
    }}
}

macro_rules! get_guess_burst_from_pos_from_param {
    (
        $param:expr
    ) => {{
        use crate::block_utils::GuessBurstFromPos;

        match $param.guess_burst_from_pos {
            Some(x) => Some(GuessBurstFromPos::NoShift(x)),
            None => match $param.from_pos {
                None => None,
                Some(x) => Some(GuessBurstFromPos::ShiftToStart(x)),
            },
        }
    }};
}

macro_rules! get_burst_or_guess {
    (
        no_offset => $param:expr, $ref_block_pos:expr, $ref_block:expr
    ) => {{
        get_burst_or_guess!($param, None, false, $ref_block_pos, $ref_block)
    }};
    (
        $param:expr, $ref_block_pos:expr, $ref_block:expr
    ) => {{
        let from_pos = get_guess_burst_from_pos_from_param!($param);

        get_burst_or_guess!($param, from_pos, $param.force_misalign, $ref_block_pos, $ref_block)
    }};
    (
        $param:expr, $from_pos:expr, $force_misalign:expr, $ref_block_pos:expr, $ref_block:expr
    ) => {{
        use crate::sbx_specs::*;
        use crate::block_utils;

        let burst = unwrap_or!($param.burst,
                               if ver_uses_rs($ref_block.get_version()) {
                                   return_if_ref_not_meta!($ref_block_pos,
                                                           $ref_block,
                                                           "guess burst error resistance level");

                                   unwrap_or!(block_utils::guess_burst_err_resistance_level(&$param.in_file,
                                                                                            $from_pos,
                                                                                            $force_misalign,
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

        print_if!(verbose not_json => $param, $param.json_printer =>
                  "Using burst error resistance level {} for the container", burst;
                  "";);

        print_field_if_json!($param.json_printer, "burst error resistance level : {}", burst);

        burst
    }}
}

macro_rules! print_if {
    (
        verbose =>
            $param:expr,
            $reporter:expr
            =>
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
    }};
    (
        verbose =>
            $param:expr
            =>
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
        json =>
            $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $printer.json_enabled() {
            print_block!( $printer.output_channel() =>
                $( $($expr),*; )*
            );
        }
    }};
    (
        not_json =>
            $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if !$printer.json_enabled() {
            print_block!( $printer.output_channel() =>
                $( $($expr),*; )*
            );
        }
    }};
    (
        verbose json =>
            $param:expr,
        $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $param.verbose && $printer.json_enabled() {
            print_block!( $printer.output_channel() =>
                $( $($expr),*; )*
            );
        }
    }};
    (
        verbose not_json =>
            $param:expr,
        $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $param.verbose && !$printer.json_enabled() {
            print_block!( $printer.output_channel() =>
                $( $($expr),*; )*
            );
        }
    }};
    (
        verbose json =>
            $param:expr,
        $reporter:expr,
        $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $param.verbose && $printer.json_enabled() {
            pause_reporter!($reporter =>
                            print_block!( $printer.output_channel() =>
                                $( $($expr),*; )*
                            );
            );
        }
    }};
    (
        verbose not_json =>
            $param:expr,
        $reporter:expr,
        $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $param.verbose && !$printer.json_enabled() {
            pause_reporter!($reporter =>
                            print_block!(
                                $( $($expr),*; )*
                            );
            );
        }
    }};
}

macro_rules! write_if {
    (
        json =>
            $f:expr,
            $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        if $printer.json_enabled() {
            write_block!($f,
                $( $($expr),*; )*
            );
        }
        Ok(())
    }};
    (
        not_json =>
            $f:expr,
            $printer:expr
            =>
            $(
                $($expr:expr),*;
            )*
    ) => {{
        use std::fmt;

        if !$printer.json_enabled() {
            write_block!($f,
                $( $($expr),*; )*
            );
        }
        let ok : fmt::Result = Ok(());
        ok
    }};
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
    }};
}

macro_rules! break_if_reached_required_len {
    (
        $bytes_processed:expr, $required_len:expr
    ) => {{
        if $bytes_processed >= $required_len {
            break;
        }
    }};
}

macro_rules! shadow_to_avoid_use {
    (
        $var:ident
    ) => {
        #[allow(unused_variables)]
        let $var = ();
    };
}

macro_rules! break_if_last {
    (
        $cur:expr, $last:expr
    ) => {{
        if $cur == $last {
            break;
        }
    }};
    (
        seq_num => $cur:expr
    ) => {{
        use crate::sbx_specs::SBX_LAST_SEQ_NUM;

        break_if_last!($cur, SBX_LAST_SEQ_NUM);
    }};
    (
        block_index => $cur:expr
    ) => {{
        break_if_last!($cur, std::u64::MAX);
    }};
}

macro_rules! incre_or_break_if_last {
    (
        $cur:expr, $last:expr
    ) => {{
        break_if_last!($cur, $last);
        $cur += 1;
    }};
    (
        seq_num => $cur:expr
    ) => {
        break_if_last!(seq_num => $cur);
        $cur += 1;
    };
    (
        block_index => $cur:expr
    ) => {
        break_if_last!(block_index => $cur);
        $cur += 1;
    };
}
