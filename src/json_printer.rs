use smallvec::SmallVec;
use json_utils::split_key_val_pair;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BracketType {
    Curly,
    Square,
}

struct JSONContext {
    first_item   : bool,
    bracket_type : BracketType,
}

pub struct JSONPrinter {
    json_enabled : bool,
    contexts     : SmallVec<[JSONContext; 8]>
}

fn print_comma_if_not_first(context : &mut JSONContext) {
    if !context.first_item {
        print!(",");
    }
    context.first_item = false;
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
    pub fn new(json_enabled : bool) -> JSONPrinter {
        JSONPrinter {
            json_enabled,
            contexts     : SmallVec::new(),
        }
    }

    pub fn json_enabled(&self) -> bool {
        self.json_enabled
    }

    pub fn first_item(&self) -> bool {
        self.contexts.last().unwrap().first_item
    }

    pub fn print_open_bracket(&mut self, bracket_type : BracketType) {
        if !self.json_enabled { return; }

        match self.contexts.last_mut() {
            None    => {},
            Some(x) => print_comma_if_not_first(x)
        }

        println!("{}", bracket_type_to_str_open(bracket_type));

        self.contexts.push(JSONContext { first_item   : true,
                                         bracket_type });
    }

    pub fn print_close_bracket(&mut self) {
        if !self.json_enabled { return; }

        let context = self.contexts.pop().unwrap();

        println!("{}", bracket_type_to_str_close(context.bracket_type));
    }

    pub fn print_maybe_json(&mut self,
                            skip_quotes : bool,
                            msg         : &str) {
        if self.json_enabled {
            let mut context = self.contexts.last_mut().unwrap();

            let (l, r) : (&str, &str) = split_key_val_pair(&msg);

            print_json_field!(l, r, skip_quotes, context.first_item);

            context.first_item = false;
        } else {
            println!("{}", msg);
        }
    }
}
