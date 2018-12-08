use smallvec::SmallVec;
use json_utils::split_key_val_pair;

use std::sync::Mutex;

use misc_utils::to_camelcase;

use std::fmt;

use output_channel::OutputChannel;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BracketType {
    Curly,
    Square,
}

#[derive(Clone, Debug)]
struct JSONContext {
    first_item   : bool,
    bracket_type : BracketType,
}

impl JSONContext {
    pub fn new(bracket_type : BracketType) -> JSONContext {
        JSONContext {
            first_item   : true,
            bracket_type,
        }
    }
}

#[derive(Debug)]
pub struct JSONPrinter {
    json_enabled   : bool,
    output_channel : OutputChannel,
    contexts       : Mutex<SmallVec<[JSONContext; 8]>>
}

impl Clone for JSONPrinter {
    fn clone(&self) -> Self {
        JSONPrinter {
            json_enabled   : self.json_enabled,
            output_channel : self.output_channel,
            contexts       : Mutex::new(self.contexts.lock().unwrap().clone())
        }
    }
}

fn print_comma_if_not_first(context : &mut JSONContext) {
    if !context.first_item {
        print!(",");
    }
    context.first_item = false;
}

fn write_comma_if_not_first(f       : &mut fmt::Formatter,
                            context : &mut JSONContext)
                            -> fmt::Result {
    if !context.first_item {
        write!(f, ",")?;
    }
    context.first_item = false;

    Ok(())
}

fn bracket_type_to_str_open(bracket_type : BracketType) -> &'static str {
    match bracket_type {
        BracketType::Curly  => "{",
        BracketType::Square => "[",
    }
}

fn bracket_type_to_str_close(bracket_type : BracketType) -> &'static str {
    match bracket_type {
        BracketType::Curly  => "}",
        BracketType::Square => "]",
    }
}

impl JSONPrinter {
    pub fn new(json_enabled : bool, output_channel : OutputChannel) -> JSONPrinter {
        JSONPrinter {
            json_enabled,
            output_channel,
            contexts     : Mutex::new(SmallVec::new()),
        }
    }

    pub fn json_enabled(&self) -> bool {
        self.json_enabled
    }

    pub fn output_channel(&self) -> OutputChannel {
        self.output_channel
    }

    pub fn set_output_channel(&mut self, output_channel : OutputChannel) {
        self.output_channel = output_channel
    }

    pub fn first_item(&self) -> bool {
        self.contexts.lock().unwrap().last().unwrap().first_item
    }

    pub fn print_open_bracket(&self,
                              name         : Option<&str>,
                              bracket_type : BracketType) {
        if !self.json_enabled { return; }

        match self.contexts.lock().unwrap().last_mut() {
            None    => {},
            Some(x) => print_comma_if_not_first(x)
        }

        match name {
            None    => {},
            Some(n) => print!("\"{}\": ", to_camelcase(n))
        }

        println!("{}", bracket_type_to_str_open(bracket_type));

        self.contexts.lock().unwrap().push(JSONContext::new(bracket_type));
    }

    pub fn write_open_bracket(&self,
                              f            : &mut fmt::Formatter,
                              name         : Option<&str>,
                              bracket_type : BracketType) -> fmt::Result {
        if !self.json_enabled { return Ok(()); }

        match self.contexts.lock().unwrap().last_mut() {
            None    => {},
            Some(x) => write_comma_if_not_first(f, x)?
        }

        match name {
            None    => {},
            Some(n) => write!(f, "\"{}\": ", to_camelcase(n))?
        }

        writeln!(f, "{}", bracket_type_to_str_open(bracket_type))?;

        self.contexts.lock().unwrap().push(JSONContext::new(bracket_type));

        Ok(())
    }

    pub fn print_close_bracket(&self) {
        if !self.json_enabled { return; }

        let context = self.contexts.lock().unwrap().pop().unwrap();

        println!("{}", bracket_type_to_str_close(context.bracket_type));
    }

    pub fn write_close_bracket(&self,
                               f : &mut fmt::Formatter) -> fmt::Result {
        if !self.json_enabled { return Ok(()); }

        let context = self.contexts.lock().unwrap().pop().unwrap();

        writeln!(f, "{}", bracket_type_to_str_close(context.bracket_type))
    }

    pub fn print_maybe_json(&self,
                            skip_quotes : bool,
                            msg         : &str) {
        if self.json_enabled {
            let mut contexts = self.contexts.lock().unwrap();
            let context  = contexts.last_mut().unwrap();

            let (l, r) : (&str, &str) = split_key_val_pair(&msg);

            print_json_field!(l, r, skip_quotes, context.first_item);

            context.first_item = false;
        } else {
            println!("{}", msg);
        }
    }

    pub fn write_maybe_json(&self,
                            f           : &mut fmt::Formatter,
                            skip_quotes : bool,
                            msg         : &str) -> fmt::Result {
        if self.json_enabled {
            let mut contexts = self.contexts.lock().unwrap();
            let context  = contexts.last_mut().unwrap();

            let (l, r) : (&str, &str) = split_key_val_pair(&msg);

            write_json_field!(f, l, r, skip_quotes, context.first_item)?;

            context.first_item = false;

            Ok(())
        } else {
            writeln!(f, "{}", msg)
        }
    }
}
