use colored::*;
use indenter::indented;
use std::fmt::{self, Debug, Display, Write};
use std::fs::read_to_string;

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};

fn main() {
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
                    println!("{}\n", process);
                }
                string => panic_event(&reader, String::from_utf8(string.to_vec()).unwrap()),
            },
            Ok(ev) => panic_event(&reader, ev),
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
            "* Reduce".red(),
            &self.original_expression.clone().unwrap_or(PoincareNode {
                name: String::new(),
                id: String::new(),
                children: Vec::new(),
                attributes: None,
            })
        )?;
        for step in &self.steps {
            write!(indented(f), "{}\n", step)?;
        }
        write!(
            f,
            "{} {}",
            "*->".red(),
            &self.result_expression.clone().unwrap_or(PoincareNode {
                name: String::new(),
                id: String::new(),
                children: Vec::new(),
                attributes: None,
            })
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
}
impl Display for StepNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let begin_str = format!("/> {} \n", self.name).bright_magenta();
        write!(f, "{}", begin_str)?;
        if let Some(before) = &self.before {
            write!(f, "{} {}\n", "|".bright_magenta(), before)?;
        }
        if self.substeps.len() > 0 {
            for substep in &self.substeps {
                write!(indented(f).with_str("|    "), "{}\n", substep)?;
            }
        }
        if let Some(after) = &self.after {
            write!(f, "{} {}", "\\".bright_magenta(), after)?;
        }
        Ok(())
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
}
impl Display for PoincareNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name, self.id)?;
        if let Some(attributes) = &self.attributes {
            let attributes_str = format!("{}", attributes).green();
            write!(f, ": {}", attributes_str)?;
        }
        if self.children.len() > 0 {
            write!(f, " {} ", "{".cyan())?;
            for child in &self.children {
                write!(f, "{}, ", child)?;
            }
            write!(f, " {}", "}".cyan())?;
        }
        Ok(())
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
                negative: get_attribute_from_start(start, b"decimal")?,
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
}
impl Display for PoincareAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasedInteger { base, integer } => write!(f, "{}__{}", integer, base)?,
            Self::CodePointLayout { code_point } => write!(f, "{}", code_point)?,
            Self::Decimal {
                negative,
                mantissa,
                exponent,
            } => write!(
                f,
                "{}{}x10^{}",
                if negative == "0" {
                    ""
                } else if negative == "1" {
                    "-"
                } else {
                    "sign?"
                },
                mantissa,
                exponent
            )?,
            Self::Float { value } => write!(f, "{}", value)?,
            Self::Infinity { negative } => write!(
                f,
                "{}inf",
                if negative == "0" {
                    ""
                } else if negative == "1" {
                    "-"
                } else {
                    "sign?"
                }
            )?,
            Self::Integer { value } => write!(f, "{}", value)?,
            Self::Matrix { rows, columns } => write!(f, "rows: {}, columns: {}", rows, columns)?,
            Self::Rational {
                negative,
                numerator,
                denominator,
            } => write!(
                f,
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
            )?,
            Self::SymbolAbstract { name } => write!(f, "{}", name)?,
            Self::Unit {
                prefix,
                root_symbol,
            } => write!(f, "{}{}", prefix, root_symbol)?,
        }
        Ok(())
    }
}
