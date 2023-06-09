use colored::*;
use std::env;
use std::fmt::Debug;
use std::fs::read_to_string;

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};

mod poincare;
mod reduction;

use reduction::{ReduceProcessNode, StepNode, StepTypeMask};

fn main() {
    let mut arguments = Arguments::from_args(env::args());
    if arguments.files.len() == 0 {
        arguments.files.push(String::from("poincare-log.xml"));
    }
    for file in &arguments.files {
        let start_file_str = format!("Reading file `{}`", file);
        println!("{}", start_file_str.red());
        let xml_string_result = read_to_string(file);
        let xml_string = match xml_string_result {
            Err(e) => {
                println!("Error while opening {}: {}", file, e);
                return;
            }
            Ok(xml_string) => xml_string,
        };
        let mut reader = Reader::from_str(&xml_string);
        reader.trim_text(true);
        loop {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(start)) => match start.name().as_ref() {
                    b"ReduceProcess" => {
                        let mut process = ReduceProcessNode::from_start(&start);
                        process.build(&mut reader);
                        if !arguments.show_useless {
                            let steps_to_remove_mask = StepTypeMask {
                                based_integer_to_rational: !arguments.show_number_to_rational,
                                to_undef: !arguments.show_to_undef,
                            };
                            StepNode::remove_useless_recursive(&mut process.steps, |step| {
                                steps_to_remove_mask.step_is_either(step)
                            });
                        }
                        println!("{}\n", process.view(arguments.print_long_form));
                    }
                    string => panic_event(&reader, String::from_utf8(string.to_vec()).unwrap()),
                },
                Ok(ev) => panic_event(&reader, ev),
            }
        }
    } // for each file
}

/// display options read from the command line
#[derive(Debug, Clone)]
struct Arguments {
    show_useless: bool,
    show_number_to_rational: bool,
    show_to_undef: bool,
    print_long_form: bool,
    // list of files to analyse
    files: Vec<String>,
}
impl Arguments {
    fn from_args(args: env::Args) -> Self {
        let mut arguments = Self::default();
        // the first argument is almost always the program name or the path it was run from
        for arg in args.skip(1) {
            let arg = arg.trim();
            match arg {
                "--useless" => arguments.show_useless = true,
                "--number-to-rational" => arguments.show_number_to_rational = true,
                "--to-undef" => arguments.show_to_undef = true,
                "--long" => arguments.print_long_form = true,
                file_name => arguments.files.push(String::from(file_name)),
            }
        }
        arguments
    }
}
impl Default for Arguments {
    fn default() -> Self {
        Self {
            show_useless: false,
            show_number_to_rational: false,
            show_to_undef: false,
            print_long_form: false,
            files: Vec::new(),
        }
    }
}

fn panic_event<T: Debug>(reader: &Reader<&[u8]>, event: T) -> ! {
    panic!(
        "Unexpected `{:?}` at position {}",
        event,
        reader.buffer_position()
    );
}

fn get_attribute_from_start(start: &BytesStart, attr_name: &[u8]) -> Option<String> {
    let mut value: Option<String> = None;
    for attr in start.attributes() {
        let attr = attr.unwrap();
        if attr.key.as_ref() == attr_name {
            value = Some(String::from_utf8(attr.value.to_vec()).unwrap());
            break;
        }
    }
    value
}
