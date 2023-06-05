use indenter::indented;
use std::fmt::{self, Debug, Display, Write};

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};

fn main() {
    let xml = r##"<ReduceProcess><OriginalExpression><Addition id="999" refCount="1" size="16"><BasedInteger id="998" refCount="1" size="28" base="10" integer="1"></BasedInteger><Multiplication id="1000" refCount="1" size="16"><BasedInteger id="1001" refCount="1" size="28" base="10" integer="3"></BasedInteger><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication><BasedInteger id="1003" refCount="1" size="28" base="10" integer="4"></BasedInteger></Addition></OriginalExpression><Step name="deepReduceChildren"><Addition id="999" refCount="1" size="16"><BasedInteger id="998" refCount="1" size="28" base="10" integer="1"></BasedInteger><Multiplication id="1000" refCount="1" size="16"><BasedInteger id="1001" refCount="1" size="28" base="10" integer="3"></BasedInteger><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication><BasedInteger id="1003" refCount="1" size="28" base="10" integer="4"></BasedInteger></Addition><Step name="deepReduceChildren"><BasedInteger id="998" refCount="2" size="28" base="10" integer="1"></BasedInteger><BasedInteger id="998" refCount="2" size="28" base="10" integer="1"></BasedInteger></Step>
    <Step name="deepReduceChildren"><Multiplication id="1000" refCount="2" size="16"><BasedInteger id="1001" refCount="1" size="28" base="10" integer="3"></BasedInteger><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication><Step name="deepReduceChildren"><BasedInteger id="1001" refCount="2" size="28" base="10" integer="3"></BasedInteger><BasedInteger id="1001" refCount="2" size="28" base="10" integer="3"></BasedInteger></Step>
    <Step name="deepReduceChildren"><Constant id="1002" refCount="2" size="19" name="π"></Constant><Constant id="1002" refCount="2" size="19" name="π"></Constant></Step>
    <Multiplication id="1000" refCount="2" size="16"><Rational id="998" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication></Step>
    <Step name="deepReduceChildren"><BasedInteger id="1003" refCount="2" size="28" base="10" integer="4"></BasedInteger><BasedInteger id="1003" refCount="2" size="28" base="10" integer="4"></BasedInteger></Step>
    <Addition id="999" refCount="1" size="16"><Rational id="1009" refCount="1" size="24" negative="0" numerator="1" denominator="1"></Rational><Multiplication id="1000" refCount="1" size="16"><Rational id="998" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication><Rational id="1001" refCount="1" size="24" negative="0" numerator="4" denominator="1"></Rational></Addition></Step>
    <Step name="shallowReduce"><Addition id="999" refCount="1" size="16"><Rational id="1009" refCount="1" size="24" negative="0" numerator="1" denominator="1"></Rational><Multiplication id="1000" refCount="1" size="16"><Rational id="998" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication><Rational id="1001" refCount="1" size="24" negative="0" numerator="4" denominator="1"></Rational></Addition><Addition id="999" refCount="2" size="16"><Rational id="1003" refCount="1" size="24" negative="0" numerator="5" denominator="1"></Rational><Multiplication id="1000" refCount="1" size="16"><Rational id="998" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1002" refCount="1" size="19" name="π"></Constant></Multiplication></Addition></Step>
    </ReduceProcess>
    <ReduceProcess><OriginalExpression><Addition id="988" refCount="1" size="16"><Multiplication id="997" refCount="1" size="16"><BasedInteger id="1003" refCount="1" size="28" base="10" integer="3"></BasedInteger><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication><BasedInteger id="991" refCount="1" size="28" base="10" integer="5"></BasedInteger></Addition></OriginalExpression><Step name="deepReduceChildren"><Addition id="988" refCount="1" size="16"><Multiplication id="997" refCount="1" size="16"><BasedInteger id="1003" refCount="1" size="28" base="10" integer="3"></BasedInteger><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication><BasedInteger id="991" refCount="1" size="28" base="10" integer="5"></BasedInteger></Addition><Step name="deepReduceChildren"><Multiplication id="997" refCount="2" size="16"><BasedInteger id="1003" refCount="1" size="28" base="10" integer="3"></BasedInteger><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication><Step name="deepReduceChildren"><BasedInteger id="1003" refCount="2" size="28" base="10" integer="3"></BasedInteger><BasedInteger id="1003" refCount="2" size="28" base="10" integer="3"></BasedInteger></Step>
    <Step name="deepReduceChildren"><Constant id="1007" refCount="2" size="19" name="π"></Constant><Constant id="1007" refCount="2" size="19" name="π"></Constant></Step>
    <Multiplication id="997" refCount="2" size="16"><Rational id="989" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication></Step>
    <Step name="deepReduceChildren"><BasedInteger id="991" refCount="2" size="28" base="10" integer="5"></BasedInteger><BasedInteger id="991" refCount="2" size="28" base="10" integer="5"></BasedInteger></Step>
    <Addition id="988" refCount="1" size="16"><Multiplication id="997" refCount="1" size="16"><Rational id="989" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication><Rational id="1003" refCount="1" size="24" negative="0" numerator="5" denominator="1"></Rational></Addition></Step>
    <Step name="shallowReduce"><Addition id="988" refCount="1" size="16"><Multiplication id="997" refCount="1" size="16"><Rational id="989" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication><Rational id="1003" refCount="1" size="24" negative="0" numerator="5" denominator="1"></Rational></Addition><Addition id="988" refCount="2" size="16"><Rational id="1003" refCount="1" size="24" negative="0" numerator="5" denominator="1"></Rational><Multiplication id="997" refCount="1" size="16"><Rational id="989" refCount="1" size="24" negative="0" numerator="3" denominator="1"></Rational><Constant id="1007" refCount="1" size="19" name="π"></Constant></Multiplication></Addition></Step>
    </ReduceProcess>
    <ReduceProcess><OriginalExpression><Decimal id="997" refCount="1" size="28" negative="0" mantissa="1442478" exponent="1"></Decimal></OriginalExpression><Step name="deepReduceChildren"><Decimal id="997" refCount="1" size="28" negative="0" mantissa="1442478" exponent="1"></Decimal><Decimal id="997" refCount="1" size="28" negative="0" mantissa="1442478" exponent="1"></Decimal></Step>
    </ReduceProcess>"##;
    
    let mut reader = Reader::from_str(xml);
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
}
impl ReduceProcessNode {
    fn from_start(_start: &BytesStart) -> Self {
        Self {
            steps: Vec::new(),
            original_expression: None,
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
            "* Reduce {}:\n",
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
        write!(f, "/> {} \n", self.name)?;
        if let Some(before) = &self.before {
            write!(f, "| {}\n", before)?;
        }
        if self.substeps.len() > 0 {
            for substep in &self.substeps {
                write!(indented(f).with_str("|    "), "{}\n", substep)?;
            }
        }
        if let Some(after) = &self.after {
            write!(f, "\\ {}", after)?;
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
            write!(f, ": {}", attributes)?;
        }
        if self.children.len() > 0 {
            write!(f, " {{ ")?;
            for child in &self.children {
                write!(f, "{}, ", child)?;
            }
            write!(f, " }}")?;
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
        #[allow(non_snake_case)]
        CodePoint: String,
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
        #[allow(non_snake_case)]
        rootSymbol: String,
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
                CodePoint: get_attribute_from_start(start, b"CodePoint")?,
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
            b"SymbolAbstract" => Some(Self::SymbolAbstract {
                name: get_attribute_from_start(start, b"name")?,
            }),
            b"Unit" => Some(Self::Unit {
                prefix: get_attribute_from_start(start, b"prefix")?,
                rootSymbol: get_attribute_from_start(start, b"rootSymbol")?,
            }),
            _ => None,
        }
    }
}
impl Display for PoincareAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasedInteger { base, integer } => write!(f, "{}<{}>", integer, base)?,
            Self::CodePointLayout { CodePoint } => write!(f, "{}", CodePoint)?,
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
            Self::Unit { prefix, rootSymbol } => write!(f, "{}{}", prefix, rootSymbol)?,
        }
        Ok(())
    }
}