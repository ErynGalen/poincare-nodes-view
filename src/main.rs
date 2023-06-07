use colored::*;
use indenter::indented;
use std::env;
use std::fmt::{self, Debug, Display, Write};
use std::fs::read_to_string;

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};

fn main() {
    let arguments = Arguments::from_args(env::args());
    let xml_string_result = read_to_string("poincare-log.xml");
    let xml_string = match xml_string_result {
        Err(e) => {
            println!("Error while opening poincare-log.xml: {}", e);
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
                        };
                        StepNode::remove_useless_recursive(&mut process.steps, |step| {
                            steps_to_remove_mask.step_is_either(step)
                        });
                    }
                    println!("{}\n", process);
                }
                string => panic_event(&reader, String::from_utf8(string.to_vec()).unwrap()),
            },
            Ok(ev) => panic_event(&reader, ev),
        }
    }
}

/// display options read from the command line
#[derive(Debug, Clone, Copy)]
struct Arguments {
    show_useless: bool,
    show_number_to_rational: bool,
}
impl Arguments {
    fn from_args(args: env::Args) -> Self {
        let mut arguments = Self::default();
        // the first argument is almost always the program name or the path it was run from
        for arg in args.skip(1) {
            let arg = arg.trim();
            if arg == "--useless" {
                arguments.show_useless = true;
            } else if arg == "--number-to-rational" {
                arguments.show_number_to_rational = true;
            } else {
                eprintln!("Unknown argument: `{}`, skipping", arg);
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

#[derive(Debug, Clone)]
struct ReduceProcessNode {
    steps: Vec<StepNode>,
    original_expression: Option<PoincareNode>,
    result_expression: Option<PoincareNode>,
}
impl ReduceProcessNode {
    fn from_start(_start: &BytesStart) -> Self {
        Self {
            steps: Vec::new(),
            original_expression: None,
            result_expression: None,
        }
    }
    fn build(&mut self, reader: &mut Reader<&[u8]>) {
        loop {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(start)) => match start.name().as_ref() {
                    b"Step" => {
                        let mut step = StepNode::from_start(&start);
                        step.build(reader);
                        self.steps.push(step);
                    }
                    b"OriginalExpression" => {
                        if self.original_expression.is_some() {
                            panic_event(reader, "Second OriginalExpression not allowed");
                        }
                        let mut expr = PoincareNode::from_previous(reader);
                        expr.build(reader);
                        self.original_expression = Some(expr);
                        match reader.read_event() {
                            Ok(Event::End(end)) => match end.name().as_ref() {
                                b"OriginalExpression" => (),
                                string => panic_event(
                                    &reader,
                                    String::from_utf8(string.to_vec()).unwrap(),
                                ),
                            },
                            other => panic_event(reader, other),
                        }
                    }
                    // TODO: (?) this is duplicated from the previous case b"OriginalExpression"
                    b"ResultExpression" => {
                        if self.result_expression.is_some() {
                            panic_event(reader, "Second ResultExpression not allowed");
                        }
                        let mut expr = PoincareNode::from_previous(reader);
                        expr.build(reader);
                        self.result_expression = Some(expr);
                        match reader.read_event() {
                            Ok(Event::End(end)) => match end.name().as_ref() {
                                b"ResultExpression" => (),
                                string => panic_event(
                                    &reader,
                                    String::from_utf8(string.to_vec()).unwrap(),
                                ),
                            },
                            other => panic_event(reader, other),
                        }
                    }
                    string => panic_event(&reader, String::from_utf8(string.to_vec()).unwrap()),
                },
                Ok(Event::End(end)) => match end.name().as_ref() {
                    b"ReduceProcess" => break,
                    string => panic_event(&reader, String::from_utf8(string.to_vec()).unwrap()),
                },
                Ok(ev) => panic_event(reader, ev),
            }
        }
    }
}
impl Display for ReduceProcessNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}:\n",
            "* Reduce".red().bold(),
            &self
                .original_expression
                .clone()
                .unwrap_or(PoincareNode {
                    name: String::new(),
                    id: String::new(),
                    children: Vec::new(),
                    attributes: None,
                })
                .pretty_print(0)
        )?;
        for step in &self.steps {
            write!(indented(f), "{}\n", step)?;
        }
        write!(
            f,
            "{} {}",
            "*->".red().bold(),
            &self
                .result_expression
                .clone()
                .unwrap_or(PoincareNode {
                    name: String::new(),
                    id: String::new(),
                    children: Vec::new(),
                    attributes: None,
                })
                .pretty_print(0)
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct StepNode {
    before: Option<PoincareNode>,
    after: Option<PoincareNode>,
    name: String,
    substeps: Vec<StepNode>,
}
impl StepNode {
    fn from_start(start: &BytesStart) -> Self {
        Self {
            before: None,
            after: None,
            name: get_attribute_from_start(start, b"name").unwrap(),
            substeps: Vec::new(),
        }
    }
    fn build(&mut self, reader: &mut Reader<&[u8]>) {
        loop {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(start)) => match start.name().as_ref() {
                    b"Step" => {
                        let mut substep = StepNode::from_start(&start);
                        substep.build(reader);
                        self.substeps.push(substep);
                    }
                    _ => {
                        let mut poincare_node =
                            PoincareNode::from_start(&start, reader.buffer_position());
                        poincare_node.build(reader);
                        if self.before.is_none() {
                            self.before = Some(poincare_node)
                        } else if self.after.is_none() {
                            self.after = Some(poincare_node)
                        } else {
                            panic_event(reader, "A step can only have two nodes");
                        }
                    }
                },
                Ok(Event::End(end)) => match end.name().as_ref() {
                    b"Step" => break,
                    string => panic_event(&reader, String::from_utf8(string.to_vec()).unwrap()),
                },
                Ok(ev) => panic_event(reader, ev),
            }
        }
    }
    /// true if the step does nothing or if it's marked as useless by `non_trivial_is_useless()`
    fn is_useful<F>(&self, non_trivial_is_useless: F) -> bool
    where
        F: Fn(&StepNode) -> bool + Copy,
    {
        if self.substeps.len() == 0 {
            if let Some(before) = &self.before {
                if let Some(after) = &self.after {
                    if before == after {
                        return false;
                    }
                    return !non_trivial_is_useless(self);
                }
            }
            // by default suppose the step is useful
            return true;
        } else {
            // at least one substep must be useful
            for substep in &self.substeps {
                if substep.is_useful(non_trivial_is_useless) {
                    return true;
                }
            }
            return false;
        }
    }
    /// removes recursively all steps which are not useful.
    /// See `is_useful()` for more information
    fn remove_useless_recursive<F>(steps: &mut Vec<StepNode>, non_trivial_is_useless: F)
    where
        F: Fn(&StepNode) -> bool + Copy,
    {
        let mut steps_to_remove: Vec<usize> = Vec::new();
        for (n, step) in steps.iter_mut().enumerate() {
            if !step.is_useful(non_trivial_is_useless) {
                steps_to_remove.push(n);
            } else {
                Self::remove_useless_recursive(&mut step.substeps, non_trivial_is_useless);
            }
        }
        // remove elements from the last one so that indexes don't change in the mean time
        while !steps_to_remove.is_empty() {
            let n = steps_to_remove.pop().unwrap();
            steps.remove(n);
        }
    }
}
impl Display for StepNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let begin_str = format!("/> {} \n", self.name).cyan().bold();
        write!(f, "{}", begin_str)?;
        if let Some(before) = &self.before {
            write!(f, "{} {}\n", "|".cyan().bold(), before.pretty_print(0))?;
        }
        if self.substeps.len() > 0 {
            for substep in &self.substeps {
                write!(indented(f).with_str("|    "), "{}\n", substep)?;
            }
        }
        if let Some(after) = &self.after {
            write!(f, "{} {}", "\\_".cyan().bold(), after.pretty_print(0))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct StepTypeMask {
    based_integer_to_rational: bool,
}
impl StepTypeMask {
    fn step_is_either(&self, step: &StepNode) -> bool {
        if self.based_integer_to_rational {
            if step.substeps.len() == 0 {
                if let Some(before) = &step.before {
                    if let Some(after) = &step.after {
                        if before.name == "BasedInteger" && after.name == "Rational" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
struct PoincareNode {
    name: String,
    id: String,
    children: Vec<PoincareNode>,
    attributes: Option<PoincareAttributes>,
}
impl PoincareNode {
    fn from_start(start: &BytesStart, pos: usize) -> Self {
        Self {
            name: String::from_utf8(start.name().as_ref().to_vec()).unwrap(),
            id: get_attribute_from_start(start, b"id").expect(&format!(
                "no id for node {} at pos {}",
                String::from_utf8(start.name().as_ref().to_vec()).unwrap(),
                pos
            )),
            attributes: PoincareAttributes::try_from_start(start),
            children: Vec::new(),
        }
    }
    fn from_previous(reader: &mut Reader<&[u8]>) -> Self {
        match reader.read_event() {
            Ok(Event::Start(start)) => Self::from_start(&start, reader.buffer_position()),
            other => panic_event(reader, other),
        }
    }
    fn build(&mut self, reader: &mut Reader<&[u8]>) {
        loop {
            match reader.read_event() {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                Ok(Event::Eof) => break,
                Ok(Event::Start(start)) => {
                    let mut child = PoincareNode::from_start(&start, reader.buffer_position());
                    child.build(reader);
                    self.children.push(child);
                }
                Ok(Event::End(end)) => {
                    if String::from_utf8(end.name().as_ref().to_vec()).unwrap() == self.name {
                        break;
                    } else {
                        panic_event(
                            reader,
                            String::from_utf8(end.name().as_ref().to_vec()).unwrap(),
                        );
                    }
                }
                Ok(ev) => panic_event(reader, ev),
            }
        }
    }
    fn pretty_print(&self, nesting_level: usize) -> ColoredString {
        let mut output = String::new();
        let id_white_str = format!("({})", self.id).white();
        output.push_str(&format!("{}{}", self.name, id_white_str));
        if let Some(attributes) = &self.attributes {
            let attributes_str = attributes.pretty_print().green();
            output.push_str(&format!(": {}", attributes_str));
        }
        if self.children.len() > 0 {
            output.push_str(" { ");
            for child in &self.children {
                output.push_str(&format!("{}, ", child.pretty_print(nesting_level + 1)));
            }
            output.push_str("}");
        }
        output.color(Self::nesting_level_color(nesting_level))
    }
    fn nesting_level_color(level: usize) -> String {
        let level = level % 3;
        String::from(match level {
            0 => "yellow",
            1 => "magenta",
            2 => "blue",
            _ => unreachable!(),
        })
    }
}
impl PartialEq for PoincareNode {
    fn eq(&self, other: &Self) -> bool {
        let lhs: u32 = self
            .id
            .parse()
            .expect("a node id should be a positive number");
        let rhs: u32 = other
            .id
            .parse()
            .expect("a node id should be a positive number");
        if lhs != rhs {
            return false;
        }
        // comparing children rely on the iterators yielding items in a well-defined order
        let lhs_children = self.children.iter();
        let rhs_children = other.children.iter();
        for (lhs_child, rhs_child) in lhs_children.zip(rhs_children) {
            if lhs_child != rhs_child {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
enum PoincareAttributes {
    BasedInteger {
        base: String,
        integer: String,
    },
    CodePointLayout {
        code_point: String,
    },
    Decimal {
        negative: String,
        mantissa: String,
        exponent: String,
    },
    Float {
        value: String,
    },
    Infinity {
        negative: String,
    },
    Integer {
        value: String,
    },
    Matrix {
        rows: String,
        columns: String,
    },
    Rational {
        negative: String,
        numerator: String,
        denominator: String,
    },
    SymbolAbstract {
        name: String,
    },
    Unit {
        prefix: String,
        root_symbol: String,
    },
}
impl PoincareAttributes {
    fn try_from_start(start: &BytesStart) -> Option<Self> {
        match start.name().as_ref() {
            // TODO: add sub classes to be recognized
            b"BasedInteger" => Some(Self::BasedInteger {
                base: get_attribute_from_start(start, b"base")?,
                integer: get_attribute_from_start(start, b"integer")?,
            }),
            b"CodePointLayout" => Some(Self::CodePointLayout {
                code_point: get_attribute_from_start(start, b"CodePoint")?,
            }),
            b"Decimal" => Some(Self::Decimal {
                negative: get_attribute_from_start(start, b"negative")?,
                mantissa: get_attribute_from_start(start, b"mantissa")?,
                exponent: get_attribute_from_start(start, b"exponent")?,
            }),
            b"Float" => Some(Self::Float {
                value: get_attribute_from_start(start, b"value")?,
            }),
            b"Infinity" => Some(Self::Infinity {
                negative: get_attribute_from_start(start, b"negative")?,
            }),
            b"Integer" => Some(Self::Integer {
                value: get_attribute_from_start(start, b"value")?,
            }),
            b"Matrix" => Some(Self::Matrix {
                rows: get_attribute_from_start(start, b"rows")?,
                columns: get_attribute_from_start(start, b"columns")?,
            }),
            b"Rational" => Some(Self::Rational {
                negative: get_attribute_from_start(start, b"negative")?,
                numerator: get_attribute_from_start(start, b"numerator")?,
                denominator: get_attribute_from_start(start, b"denominator")?,
            }),
            b"SymbolAbstract"
            // Subclasses of SymbolAbstract
            | b"Symbol" | b"Sequence" | b"Function" | b"Constant" => {
                Some(Self::SymbolAbstract {
                    name: get_attribute_from_start(start, b"name")?,
                })
            }
            b"Unit" => Some(Self::Unit {
                prefix: get_attribute_from_start(start, b"prefix")?,
                root_symbol: get_attribute_from_start(start, b"rootSymbol")?,
            }),
            _ => None,
        }
    }
    fn pretty_print(&self) -> String {
        match self {
            Self::BasedInteger { base, integer } => format!("{}__{}", integer, base),
            Self::CodePointLayout { code_point } => format!("{}", code_point),
            Self::Decimal {
                negative,
                mantissa,
                exponent,
            } => format!(
                "{}{} x10^{}",
                if negative == "0" {
                    ""
                } else if negative == "1" {
                    "-"
                } else {
                    "sign?"
                },
                mantissa,
                exponent
            ),
            Self::Float { value } => format!("{}", value),
            Self::Infinity { negative } => format!(
                "{}inf",
                if negative == "0" {
                    ""
                } else if negative == "1" {
                    "-"
                } else {
                    "sign?"
                }
            ),
            Self::Integer { value } => format!("{}", value),
            Self::Matrix { rows, columns } => format!("rows: {}, columns: {}", rows, columns),
            Self::Rational {
                negative,
                numerator,
                denominator,
            } => format!(
                "{}{}/{}",
                if negative == "0" {
                    ""
                } else if negative == "1" {
                    "-"
                } else {
                    "sign?"
                },
                numerator,
                denominator
            ),
            Self::SymbolAbstract { name } => format!("{}", name),
            Self::Unit {
                prefix,
                root_symbol,
            } => format!("{}{}", prefix, root_symbol),
        }
    }
}
