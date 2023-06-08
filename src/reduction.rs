use colored::*;
use std::fmt::{self, Debug, Display, Write};

use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};

use indenter::indented;

use crate::poincare::PoincareNode;
use crate::{get_attribute_from_start, panic_event};

#[derive(Debug, Clone)]
pub struct ReduceProcessNode {
    pub steps: Vec<StepNode>,
    pub original_expression: Option<PoincareNode>,
    pub result_expression: Option<PoincareNode>,
}
impl ReduceProcessNode {
    pub fn from_start(_start: &BytesStart) -> Self {
        Self {
            steps: Vec::new(),
            original_expression: None,
            result_expression: None,
        }
    }
    pub fn build(&mut self, reader: &mut Reader<&[u8]>) {
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
                .pretty_print(0, false)
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
                .pretty_print(0, false)
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StepNode {
    pub before: Option<PoincareNode>,
    pub after: Option<PoincareNode>,
    pub name: String,
    pub substeps: Vec<StepNode>,
}
impl StepNode {
    pub fn from_start(start: &BytesStart) -> Self {
        Self {
            before: None,
            after: None,
            name: get_attribute_from_start(start, b"name").unwrap(),
            substeps: Vec::new(),
        }
    }
    pub fn build(&mut self, reader: &mut Reader<&[u8]>) {
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
    pub fn is_useful<F>(&self, non_trivial_is_useless: F) -> bool
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
    pub fn remove_useless_recursive<F>(steps: &mut Vec<StepNode>, non_trivial_is_useless: F)
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
            write!(f, "{} {}\n", "|".cyan().bold(), before.pretty_print(0, false))?;
        }
        if self.substeps.len() > 0 {
            for substep in &self.substeps {
                write!(indented(f).with_str("|    "), "{}\n", substep)?;
            }
        }
        if let Some(after) = &self.after {
            write!(f, "{} {}", "\\_".cyan().bold(), after.pretty_print(0, false))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StepTypeMask {
    pub based_integer_to_rational: bool,
}
impl StepTypeMask {
    pub fn step_is_either(&self, step: &StepNode) -> bool {
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
