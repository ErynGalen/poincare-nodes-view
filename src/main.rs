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

use reduction::{StepNode, StepPart, StepTypeMask};

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
                let error_str = format!("{}", e);
                println!("Error while opening `{}`: {}", file, error_str.red());
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
                    b"Step" => {
                        let mut step = StepNode::from_start(&start);
                        step.build(&mut reader);
                        if !arguments.show_useless {
                            let steps_to_remove_mask = StepTypeMask {
                                based_integer_to_rational: !arguments.show_number_to_rational,
                                to_undef: !arguments.show_to_undef,
                            };
                            StepPart::remove_useless_recursive(
                                &mut step.parts,
                                |part| match part {
                                    StepPart::State(..) => arguments.dont_show_intermediate_states,
                                    StepPart::Substep(step) => {
                                        step.does_nothing() || steps_to_remove_mask.step_is_either(step)
                                    }
                                },
                            );
                        }
                        println!("{}\n", step.view(arguments.print_long_form));
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
    dont_show_intermediate_states: bool,
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
                "--no-states" => arguments.dont_show_intermediate_states = true,
                file_name if !file_name.starts_with("--") => arguments.files.push(String::from(file_name)),
                opt  => eprintln!("Unknown option: '{}', skipping", opt),
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
            dont_show_intermediate_states: false,
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
