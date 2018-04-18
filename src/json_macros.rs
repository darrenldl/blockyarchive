macro_rules! write_json_field {
    (
        $f:expr, $key:expr, $val:expr, $skip_quotes:expr, $no_comma:expr
    ) => {{
        use misc_utils::escape_quotes;

        if !$no_comma {
            write!($f, ",")?;
        }

        if $skip_quotes {
            writeln!($f, "\"{}\": {}", $key, escape_quotes(&$val))
        } else {
            writeln!($f, "\"{}\": \"{}\"", $key, escape_quotes(&$val))
        }
    }};
}

macro_rules! print_json_field {
    (
        $key:expr, $val:expr, $skip_quotes:expr, $no_comma:expr
    ) => {{
        use misc_utils::{escape_quotes,
                         to_camelcase};

        if !$no_comma {
            print!(",");
        }

        if $skip_quotes {
            println!("\"{}\": {}", to_camelcase($key), escape_quotes(&$val));
        } else {
            println!("\"{}\": \"{}\"", to_camelcase($key), escape_quotes(&$val));
        }
    }};
}

macro_rules! print_maybe_json_open_bracket {
    (
        $json_context:expr
    ) => {{
        if $json_context.json_enabled {
            println!("{{");
        }
    }}
}

macro_rules! print_maybe_json_close_bracket {
    (
        $json_context:expr
    ) => {{
        if $json_context.json_enabled {
            println!("}}");
        }
    }}
}

macro_rules! print_if_not_json {
    (
        $json_printer:expr, $($val:expr),*
    ) => {{
        if !$json_printer.json_enabled() {
            println!($($val),*);
        }
    }}
}

macro_rules! print_bracket {
    (
        $json_context:expr => open curly
    ) => {{
        if $json_context.json_enabled {
            if !$json_context.first_item {
                print!(",");
            }
            print!("{{")
        }
    }};
    (
        $json_context:expr => close curly
    ) => {{
        if $json_context.json_enabled {
            print!("}}")
        }
    }};
    (
        $json_context:expr => open square
    ) => {{
        if $json_context.json_enabled {
            if !$json_context.first_item {
                print!(",");
            }
            print!("[")
        }
    }};
    (
        $json_context:expr => close square
    ) => {{
        if $json_context.json_enabled {
            print!("]")
        }
    }};
}

macro_rules! print_if_json {
    (
        $json_context:expr, $($val:expr),*
    ) => {{
        if $json_context.json_enabled {
            println!($($val),*);
        }
    }}
}

macro_rules! print_field_if_json {
    (
        $json_context:expr, $($t:tt)*
    ) => {{
        if $json_context.json_enabled {
            print_maybe_json!($json_context, $($t)*);
        }
    }}
}

macro_rules! print_maybe_json {
    (
        $json_printer:expr, $($val:expr),* => skip_quotes
    ) => {{
        print_maybe_json!($json_printer, $($val),* => true)
    }};
    (
        $json_printer:expr, $($val:expr),*
    ) => {{
        print_maybe_json!($json_printer, $($val),* => false)
    }};
    (
        $json_printer:expr, $($val:expr),* => $skip_quotes:expr
    ) => {{
        let msg = format!($($val),*);

        $json_printer.print_maybe_json($skip_quotes, &msg);
    }}
}

macro_rules! write_maybe_json {
    (
        $f:expr, $json_context:expr, $($val:expr),* => skip_quotes
    ) => {{
        write_maybe_json!($f, $json_context, $($val),* => true)
    }};
    (
        $f:expr, $json_context:expr, $($val:expr),*
    ) => {{
        write_maybe_json!($f, $json_context, $($val),* => false)
    }};
    (
        $f:expr, $json_context:expr, $($val:expr),* => $skip_quotes:expr
    ) => {{
        use misc_utils::to_camelcase;
        use json_utils::split_key_val_pair;

        let res = if $json_context.json_enabled {
            let msg = format!($($val),*);

            let (l, r) : (&str, &str) = split_key_val_pair(&msg);

            write_json_field!($f, to_camelcase(l), r, $skip_quotes, $json_context.first_item)
        } else {
            writeln!($f, $($val),*)
        };

        $json_context.first_item = false;

        res
    }}
}

macro_rules! null_if_json_else {
    (
        $json_printer:expr, $val:expr
    ) => {{
        if $json_printer.json_enabled() {
            "null"
        } else {
            $val
        }
    }}
}
