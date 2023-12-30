pub mod expression;

use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
};

use expression::{WgslExpression, WgslLiteral};

#[derive(Debug)]
pub enum WgslSegmentEndReason {
    None,
    EndOfFile,
    ElseOp,
    EndOp,
}

#[derive(Debug, Clone)]
pub enum WgslSegment {
    Include(PathBuf),
    Conditional {
        condition: WgslExpression,
        if_true: Box<WgslSegment>,
        if_false: Option<Box<WgslSegment>>,
    },
    Sequence(Vec<WgslSegment>),
    Constant(String),
    Text(String),
}

impl WgslSegment {
    pub fn write(&self, output: &mut String, workspace: &WgslWorkspace) -> Result<(), WgslError> {
        match self {
            WgslSegment::Include(i) => {
                output.push_str(&workspace.get_shader(i)?);
                output.push('\n');
            }
            WgslSegment::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                let is_true = match condition.evaluate(workspace.state())? {
                    WgslLiteral::Integer(i) => i != 0,
                    WgslLiteral::Float(f) => f != 0.0,
                    WgslLiteral::Bool(b) => b,
                };

                if is_true {
                    if_true.write(output, workspace)?;
                } else if let Some(if_false) = if_false.as_ref() {
                    if_false.write(output, workspace)?;
                }
            }
            WgslSegment::Sequence(sequence) => {
                for segment in sequence.iter() {
                    segment.write(output, workspace)?;
                }
            }
            WgslSegment::Constant(name) => {
                let value = workspace
                    .state()
                    .get(name)
                    .ok_or(WgslError::UndefinedVariable)?;

                match value {
                    WgslLiteral::Integer(i) => output.push_str(&format!("const {name} = {i};\n")),
                    WgslLiteral::Float(f) => output.push_str(&format!("const {name} = {f};\n")),
                    WgslLiteral::Bool(b) => output.push_str(&format!("const {name} = {b};\n")),
                }
            }
            WgslSegment::Text(t) => output.push_str(t),
        }

        Ok(())
    }

    pub fn from_lines<'a>(
        lines: &mut impl Iterator<Item = &'a str>,
    ) -> Result<(Option<Self>, WgslSegmentEndReason), WgslError> {
        let mut segment = WgslSegment::Text(String::new());

        while let Some(line) = lines.next() {
            let line = line.trim();

            if !line.starts_with("//:") {
                segment.concat(Self::Text(format!("{line}\n")));
                continue;
            }

            let line = line[3..].to_owned();

            let (operation, parameter) = line.split_once(' ').unwrap_or((&line, ""));

            match operation {
                "include" => segment.concat(WgslSegment::Include(parameter.into())),
                "const" => segment.concat(WgslSegment::Constant(parameter.into())),
                "if" => {
                    let condition = WgslExpression::new(parameter)?;

                    let (if_true, if_false) = match WgslSegment::from_lines(lines)? {
                        (Some(segment), WgslSegmentEndReason::ElseOp) => (
                            Box::new(segment),
                            Some(Box::new(
                                WgslSegment::from_lines(lines)?
                                    .0
                                    .ok_or(WgslError::InvalidIfBlock)?,
                            )),
                        ),
                        (
                            Some(segment),
                            WgslSegmentEndReason::EndOp | WgslSegmentEndReason::EndOfFile,
                        ) => (Box::new(segment), None),
                        _ => Err(WgslError::InvalidIfBlock)?,
                    };

                    segment.concat(WgslSegment::Conditional {
                        condition,
                        if_true,
                        if_false,
                    });
                }
                "else" => return Ok((Some(segment), WgslSegmentEndReason::ElseOp)),
                "end" => return Ok((Some(segment), WgslSegmentEndReason::EndOp)),
                other => Err(WgslError::UnknownOperation(other.to_string()))?,
            }
        }

        Ok((Some(segment), WgslSegmentEndReason::EndOfFile))
    }

    #[inline]
    pub fn can_concat_fast(&self, other: &WgslSegment) -> bool {
        matches!(
            (self, other),
            (_, WgslSegment::Sequence(_))
                | (WgslSegment::Sequence(_), _)
                | (WgslSegment::Text(_), WgslSegment::Text(_))
        )
    }

    pub fn concat(&mut self, other: WgslSegment) {
        match (self, other) {
            (WgslSegment::Sequence(left), WgslSegment::Sequence(mut right)) => {
                left.reserve(right.len());

                for segment in right.drain(..) {
                    if left.last().is_some_and(|l| l.can_concat_fast(&segment)) {
                        unsafe { left.last_mut().unwrap_unchecked() }.concat(segment);
                    } else {
                        left.push(segment);
                    }
                }
            }
            (WgslSegment::Text(left), WgslSegment::Text(right)) => {
                left.push_str(&right);
            }
            (left, mut right @ WgslSegment::Sequence(_)) => {
                // The following is a safe version of:
                // right.insert(0, left);
                // left = right;

                core::mem::swap(left, &mut right);

                // Values now swapped so we swap names as well
                let (left, right) = (right, left);

                match right {
                    WgslSegment::Sequence(sequence) => sequence.insert(0, left),
                    _ => unreachable!(),
                }
            }
            (WgslSegment::Sequence(ref mut sequence), right) => match sequence.last_mut() {
                Some(segment) if segment.can_concat_fast(&right) => segment.concat(right),
                _ => sequence.push(right),
            },
            (left, right) => {
                // The following is basically a version of this but written in
                // a way that the Rust compiler won't complain:
                // *self = WgslSegment::Sequence(vec![left, right])

                let mut sequence = WgslSegment::Sequence(Vec::new());
                core::mem::swap(left, &mut sequence);

                // Values now swapped so we swap names as well
                let (sequence, left) = (left, sequence);

                match sequence {
                    WgslSegment::Sequence(ref mut sequence) => {
                        sequence.push(left);
                        sequence.push(right);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WgslShader {
    segment: WgslSegment,
    capacity: usize,
}

impl WgslShader {
    pub fn new(source: &str) -> Result<Self, WgslError> {
        let capacity = source.len();
        let mut lines = source.lines().filter(|l| !l.trim().is_empty());
        let segment = WgslSegment::from_lines(&mut lines)?
            .0
            .unwrap_or(WgslSegment::Text(String::new()));

        if lines.clone().next().is_some() {
            Err(WgslError::LeftoverChars(lines.collect()))?;
        }

        Ok(Self { segment, capacity })
    }

    fn evaluate(&self, workspace: &WgslWorkspace) -> Result<String, WgslError> {
        let mut result = String::with_capacity(self.capacity);

        self.segment.write(&mut result, workspace)?;

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct WgslWorkspaceState {
    global_variables: HashMap<String, WgslLiteral>,
    local_overrides: HashMap<String, WgslLiteral>,
}

impl WgslWorkspaceState {
    pub fn get(&self, key: &str) -> Option<WgslLiteral> {
        self.local_overrides
            .get(key)
            .copied()
            .or(self.global_variables.get(key).copied())
    }
}

impl Default for WgslWorkspaceState {
    fn default() -> Self {
        let local_overrides = HashMap::new();
        let mut global_variables = HashMap::new();

        for i in 0..64 {
            global_variables.insert(format!("BIT_{i}"), WgslLiteral::Integer(1 << i));
        }

        Self {
            global_variables,
            local_overrides,
        }
    }
}

pub struct WgslWorkspace {
    state: WgslWorkspaceState,
    root: PathBuf,
    shaders: HashMap<PathBuf, WgslShader>,
}

impl WgslWorkspace {
    pub fn scan(root: impl Into<PathBuf>) -> Self {
        let mut shaders = HashMap::new();

        Self {
            state: WgslWorkspaceState::default(),
            root: root.into(),
            shaders,
        }
    }

    /// - `root`: The root of the workspace
    /// - `shaders`: A list of shaders `(path, source)`, path is relative to
    /// `root`
    pub fn from_memory(
        root: impl Into<PathBuf>,
        shaders: &[(&str, &str)],
    ) -> Result<Self, WgslError> {
        let shaders = shaders
            .iter()
            .map(|(path, source)| Ok((path.into(), WgslShader::new(source)?)))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            state: WgslWorkspaceState::default(),
            root: root.into(),
            shaders,
        })
    }

    pub fn set_global_i64(&mut self, key: &str, value: i64) {
        self.state
            .global_variables
            .insert(key.to_string(), WgslLiteral::Integer(value));
    }

    pub fn set_global_f64(&mut self, key: &str, value: f64) {
        self.state
            .global_variables
            .insert(key.to_string(), WgslLiteral::Float(value));
    }

    pub fn set_global_bool(&mut self, key: &str, value: bool) {
        self.state
            .global_variables
            .insert(key.to_string(), WgslLiteral::Bool(value));
    }

    fn state(&self) -> &WgslWorkspaceState {
        &self.state
    }

    pub fn get_shader(&self, path: impl Into<PathBuf>) -> Result<String, WgslError> {
        self.shaders
            .get(&path.into())
            .ok_or(WgslError::NotFound)?
            .evaluate(self)
    }
}

#[derive(Debug, Clone)]
pub enum WgslError {
    UnknownOperation(String),
    InvalidIfBlock,
    NoExpression,
    NoClosingParenthesis,
    DuplicatePeriod,
    InvalidBase,
    ParseFloatError(ParseFloatError),
    ParseIntError(ParseIntError),
    LeftoverChars(String),
    UndefinedVariable,
    InvalidExpression,
    NotFound,
}
