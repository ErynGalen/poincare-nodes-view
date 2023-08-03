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
pub struct StepNode {
    pub before: Option<PoincareNode>,
    pub after: Option<PoincareNode>,
    pub parts: Vec<StepPart>,
    pub name: String,
}
impl StepNode {
    pub fn from_start(start: &BytesStart) -> Self {
        Self {
            before: None,
            after: None,
            parts: Vec::new(),
            name: get_attribute_from_start(start, b"name").expect("A step must have a name"),
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
                        self.parts.push(StepPart::Substep(substep));
                    }
                    b"State" => {
                        let mut poincare_node = PoincareNode::from_previous(reader);
                        poincare_node.build(reader);
                        let state_name = get_attribute_from_start(&start, b"name");
                        match state_name {
                            Some(name) if name == "before" => self.before = Some(poincare_node),
                            Some(name) if name == "after" => self.after = Some(poincare_node),
                            name => self.parts.push(StepPart::State(name, poincare_node)),
                        }
                        match reader.read_event() {
                            Ok(Event::End(end)) => match end.name().as_ref() {
                                b"State" => (),
                                string => panic_event(
                                    reader,
                                    String::from_utf8(string.to_vec()).unwrap(),
                                ),
                            },
                            other => panic_event(reader, other),
                        }
                    }
                    start => panic_event(reader, String::from_utf8(start.to_vec()).unwrap()),
                },
                Ok(Event::End(end)) => match end.name().as_ref() {
                    b"Step" => break,
                    string => panic_event(reader, String::from_utf8(string.to_vec()).unwrap()),
                },
                Ok(ev) => panic_event(reader, ev),
            }
        }
    }
    /// true if the step does nothing or if it's marked as useless by `non_trivial_is_useless()`
    pub fn does_nothing(&self) -> bool {
        if let Some(before) = &self.before {
            if let Some(after) = &self.after {
                if before == after {
                    return true;
                }
            }
        }
        // by default suppose the step does something
        false
    }
    pub fn view(&self, long_form: bool) -> StepView {
        StepView {
            node: self,
            long_form,
        }
    }
}
#[derive(Debug, Clone)]
pub struct StepView<'a> {
    node: &'a StepNode,
    long_form: bool,
}
impl<'a> Display for StepView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let begin_str = format!("/> {} \n", self.node.name).cyan().bold();
        write!(f, "{}", begin_str)?;
        if let Some(before) = &self.node.before {
            writeln!(
                f,
                "{} {}",
                "|".cyan().bold(),
                before.pretty_print(0, self.long_form)
            )?;
        }
        if self.node.parts.len() > 0 {
            for part in &self.node.parts {
                match part {
                    StepPart::State(name, state) => {
                        let state_prefix_str = if let Some(name) = name {
                            format!("{}: ", name)
                        } else {
                            String::new()
                        };
                        writeln!(
                            f,
                            "{}{}{}",
                            "|- ".cyan().bold(),
                            state_prefix_str.cyan(),
                            state.pretty_print(0, self.long_form)
                        )?;
                    }
                    StepPart::Substep(substep) => writeln!(
                        indented(f).with_str("|    "),
                        "{}",
                        substep.view(self.long_form)
                    )?,
                }
            }
        }
        if let Some(after) = &self.node.after {
            write!(
                f,
                "{} {}",
                "\\_".cyan().bold(),
                after.pretty_print(0, self.long_form)
            )?;
        } else {
            write!(f, "{}", "\\_".cyan().bold())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum StepPart {
    State(Option<String>, PoincareNode),
    Substep(StepNode),
}
impl StepPart {
    /// removes recursively all step parts which are not useful, according to `is_useless`
    pub fn remove_useless_recursive<F>(steps: &mut Vec<StepPart>, is_useless: F)
    where
        F: Fn(&StepPart) -> bool + Copy,
    {
        fn is_useless_recursive<F>(part: &StepPart, is_useless_shallow: F) -> bool
        where
            F: Fn(&StepPart) -> bool + Copy,
        {
            if is_useless_shallow(part) {
                return true;
            }
            // if there are substeps, at least one must be useful
            if let StepPart::Substep(step) = part {
                if step.parts.len() > 0 {
                    for part in &step.parts {
                        if !is_useless_recursive(part, is_useless_shallow) {
                            return false;
                        }
                    }
                    return true; // all substeps were useless
                }
            }
            false // assume a step is useful by default
        }
        let mut parts_to_remove: Vec<usize> = Vec::new();
        for (n, part) in steps.iter_mut().enumerate() {
            if is_useless_recursive(part, is_useless) {
                parts_to_remove.push(n);
            } else {
                match part {
                    StepPart::State(..) => (), // keep a useful state
                    StepPart::Substep(step) => {
                        Self::remove_useless_recursive(&mut step.parts, is_useless)
                    }
                };
            }
        }
        // remove elements from the last one so that indexes don't change in the mean time
        while !parts_to_remove.is_empty() {
            let n = parts_to_remove.pop().unwrap();
            steps.remove(n);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StepTypeMask {
    pub based_integer_to_rational: bool,
    pub to_undef: bool,
}
impl StepTypeMask {
    pub fn step_is_either(&self, step: &StepNode) -> bool {
        if self.based_integer_to_rational {
            if step.parts.len() == 0 {
                if let Some(before) = &step.before {
                    if let Some(after) = &step.after {
                        if before.name == "BasedInteger" && after.name == "Rational" {
                            return true;
                        }
                    }
                }
            }
        }
        if self.to_undef {
            if let Some(result) = &step.after {
                fn node_is_undef(node: &PoincareNode) -> bool {
                    if node.name == "Undefined" {
                        return true;
                    }
                    for child in &node.children {
                        if node_is_undef(child) {
                            return true;
                        }
                    }
                    false
                }
                if node_is_undef(result) {
                    return true;
                }
            }
        }
        false
    }
}
impl Default for StepTypeMask {
    fn default() -> Self {
        Self {
            based_integer_to_rational: false,
            to_undef: false,
        }
    }
}
