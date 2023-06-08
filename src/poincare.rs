use colored::*;

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};

use crate::{get_attribute_from_start, panic_event};

#[derive(Debug, Clone)]
pub struct PoincareNode {
    pub name: String,
    pub id: String,
    pub children: Vec<PoincareNode>,
    pub attributes: Option<PoincareAttributes>,
}
impl PoincareNode {
    pub fn from_start(start: &BytesStart, pos: usize) -> Self {
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
    pub fn from_previous(reader: &mut Reader<&[u8]>) -> Self {
        match reader.read_event() {
            Ok(Event::Start(start)) => Self::from_start(&start, reader.buffer_position()),
            other => panic_event(reader, other),
        }
    }
    pub fn build(&mut self, reader: &mut Reader<&[u8]>) {
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
    fn print_long_form(&self, nesting_level: usize, long_form_for_children: bool) -> ColoredString {
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
                output.push_str(&format!(
                    "{}, ",
                    child.pretty_print(nesting_level + 1, long_form_for_children)
                ));
            }
            output.push_str("}");
        }
        output.color(Self::nesting_level_color(nesting_level))
    }
    pub fn pretty_print(&self, nesting_level: usize, long_form: bool) -> ColoredString {
        if long_form {
            return self.print_long_form(nesting_level, long_form);
        }
        let mut output = String::new();
        let mut children_output: Vec<ColoredString> = Vec::new();
        if self.children.len() > 0 {
            for child in &self.children {
                children_output.push(child.pretty_print(nesting_level + 1, long_form));
            }
        }
        'types: {
            // TODO: abstract this
            match self.name.as_str() {
                // only display attributes for these nodes
                "Symbol" | "SymbolAbstract" | "Sequence" | "Function" | "Constant"
                | "BasedInteger" | "Decimal" | "Float" | "Integer" | "Rational" => {
                    if let Some(attr) = &self.attributes {
                        output.push_str(&attr.pretty_print());
                        break 'types;
                    }
                }
                _ => (),
            }
            // TODO: abstract this
            let nary_operation: Option<&str> = match self.name.as_str() {
                "Addition" => Some("+"),
                "Subtraction" => Some("-"),
                "Multiplication" => Some("*"),
                "Division" => Some("/"),
                "Power" => Some("^"),
                _ => None,
            };
            if let Some(op) = nary_operation {
                let mut child_n = 0;
                for child_str in &children_output {
                    // only add parentheses when the child has more than one children
                    let child_str = if child_n == children_output.len() - 1 {
                        if self.children[child_n].children.len() > 1 {
                            format!("({})", &child_str)
                        } else {
                            format!("{}", &child_str)
                        }
                    } else {
                        if self.children[child_n].children.len() > 1 {
                            format!("({}) {} ", child_str, op)
                        } else {
                            format!("{} {} ", child_str, op)
                        }
                    };
                    output.push_str(&child_str);
                    child_n += 1;
                }
                break 'types;
            }
            let prefixed_function: Option<&str> = match self.name.as_str() {
                // TODO: complete the list
                "AbsoluteValue" => Some("abs"),
                "ArcCosine" => Some("acos"),
                "ArcSine" => Some("asin"),
                "ArcTangent" => Some("atan"),
                "BinomCDF" => Some("bCDF"),
                "BinomPDF" => Some("bPDF"),
                "Ceiling" => Some("ceil"),
                "Conjugate" => Some("conj"),
                "Cosine" => Some("cos"),
                "Derivative" => Some("der"),
                "Floor" => Some("floor"),
                "FracPart" => Some("frac"),
                "GreatCommonDivisor" => Some("gcd"),
                "HyperbolicArcCosine" => Some("hacos"),
                "HyperbolicArcSine" => Some("hasin"),
                "HyperbolicArcTangent" => Some("hatan"),
                "HyperbolicCosine" => Some("hcos"),
                "HyperbolicSine" => Some("hsin"),
                "HyperbolicTangent" => Some("htan"),
                "ImaginaryPart" => Some("imag"),
                "LeastCommonMultiple" => Some("lcm"),
                "Integral" => Some("int"),
                "Logarithm" => Some("log"),
                "Opposite" => Some("-"),
                "Randint" => Some("randint"),
                "Random" => Some("rand"),
                "RealPart" => Some("real"),
                "Round" => Some("round"),
                "SignFunction" => Some("sign"),
                "Sine" => Some("sin"),
                "Tangent" => Some("tan"),
                "SquareRoot" => Some("sqrt"),
                "NaperianLogarithm" => Some("ln"),
                _ => None,
            };
            if let Some(f) = prefixed_function {
                let mut child_n = 0;
                for child_str in &children_output {
                    if child_n == 0 {
                        output.push_str(&format!("{}({}", f, child_str));
                    } else {
                        output.push_str(&format!(", {}", child_str));
                    }
                    if child_n == children_output.len() - 1 {
                        output.push_str(")");
                    }
                    child_n += 1;
                }
                break 'types;
            }
            let plain_name: Option<&str> = match self.name.as_str() {
                "Undefined" => Some("undef"),
                _ => None,
            };
            if let Some(name) = plain_name {
                output.push_str(name);
                break 'types;
            }
            if self.name == "Parenthesis" {
                if self.children.len() != 1 {
                    panic!("ParenthesisNode can only have exactly one child");
                }
                // display {} for ParenthesisNode to differentiate it from other parentheses
                output.push_str(&format!("{{{}}}", children_output.first().unwrap()));
                break 'types;
            }
            // default to full log when nothing else is available
            output.push_str(&self.print_long_form(nesting_level, false));
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
pub enum PoincareAttributes {
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
    pub fn try_from_start(start: &BytesStart) -> Option<Self> {
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
    pub fn pretty_print(&self) -> String {
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
