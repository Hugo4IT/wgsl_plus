use crate::{WgslError, WgslWorkspaceState};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum WgslLiteral {
    Integer(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum WgslOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    BitwiseAnd,
    BitwiseOr,
}

impl WgslOperator {
    fn priority(&self) -> usize {
        match self {
            Self::Add => 0,
            Self::Subtract => 1,
            Self::Multiply => 2,
            Self::Divide => 3,
            Self::BitwiseAnd => 4,
            Self::BitwiseOr => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WgslUnaryOperator {
    Negate,
    Not,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy)]
pub enum WgslComparison {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum WgslExpression {
    Literal(WgslLiteral),
    Reference(String),
    Operator {
        left: Box<WgslExpression>,
        operator: WgslOperator,
        right: Box<WgslExpression>,
    },
    Unary {
        operator: WgslUnaryOperator,
        right: Box<WgslExpression>,
    },
    Comparison {
        left: Box<WgslExpression>,
        comparison: WgslComparison,
        right: Box<WgslExpression>,
    },
    Parenthesized(Box<WgslExpression>),
}

impl WgslExpression {
    pub fn new(source: &str) -> Result<Self, WgslError> {
        let mut chars = source.trim().chars().filter(|c| !c.is_whitespace());
        let mut output =
            Self::from_chars(&mut chars, false).map(|r| r.ok_or(WgslError::NoExpression))??;

        output.reorder();

        if chars.clone().next().is_some() {
            Err(WgslError::LeftoverChars(chars.collect()))?
        } else {
            Ok(output)
        }
    }

    pub fn evaluate(&self, state: &WgslWorkspaceState) -> Result<WgslLiteral, WgslError> {
        match self {
            WgslExpression::Literal(l) => Ok(*l),
            WgslExpression::Reference(r) => state.get(r).ok_or(WgslError::UndefinedVariable),
            WgslExpression::Operator {
                left,
                operator,
                right,
            } => {
                let left = left.evaluate(state)?;
                let right = right.evaluate(state)?;

                match operator {
                    WgslOperator::Add => match (left, right) {
                        (WgslLiteral::Integer(left), WgslLiteral::Integer(right)) => {
                            Ok(WgslLiteral::Integer(left + right))
                        }
                        (WgslLiteral::Float(left), WgslLiteral::Float(right)) => {
                            Ok(WgslLiteral::Float(left + right))
                        }
                        _ => Err(WgslError::InvalidExpression),
                    },
                    WgslOperator::Subtract => match (left, right) {
                        (WgslLiteral::Integer(left), WgslLiteral::Integer(right)) => {
                            Ok(WgslLiteral::Integer(left - right))
                        }
                        (WgslLiteral::Float(left), WgslLiteral::Float(right)) => {
                            Ok(WgslLiteral::Float(left - right))
                        }
                        _ => Err(WgslError::InvalidExpression),
                    },
                    WgslOperator::Multiply => match (left, right) {
                        (WgslLiteral::Integer(left), WgslLiteral::Integer(right)) => {
                            Ok(WgslLiteral::Integer(left * right))
                        }
                        (WgslLiteral::Float(left), WgslLiteral::Float(right)) => {
                            Ok(WgslLiteral::Float(left * right))
                        }
                        _ => Err(WgslError::InvalidExpression),
                    },
                    WgslOperator::Divide => match (left, right) {
                        (WgslLiteral::Integer(left), WgslLiteral::Integer(right)) => {
                            Ok(WgslLiteral::Integer(left / right))
                        }
                        (WgslLiteral::Float(left), WgslLiteral::Float(right)) => {
                            Ok(WgslLiteral::Float(left / right))
                        }
                        _ => Err(WgslError::InvalidExpression),
                    },
                    WgslOperator::BitwiseAnd => match (left, right) {
                        (WgslLiteral::Integer(left), WgslLiteral::Integer(right)) => {
                            Ok(WgslLiteral::Integer(left & right))
                        }
                        (WgslLiteral::Bool(left), WgslLiteral::Bool(right)) => {
                            Ok(WgslLiteral::Bool(left & right))
                        }
                        _ => Err(WgslError::InvalidExpression),
                    },
                    WgslOperator::BitwiseOr => match (left, right) {
                        (WgslLiteral::Integer(left), WgslLiteral::Integer(right)) => {
                            Ok(WgslLiteral::Integer(left | right))
                        }
                        (WgslLiteral::Bool(left), WgslLiteral::Bool(right)) => {
                            Ok(WgslLiteral::Bool(left | right))
                        }
                        _ => Err(WgslError::InvalidExpression),
                    },
                }
            }
            WgslExpression::Unary { operator, right } => {
                let right = right.evaluate(state)?;

                match (operator, right) {
                    (WgslUnaryOperator::Negate, WgslLiteral::Integer(i)) => {
                        Ok(WgslLiteral::Integer(-i))
                    }
                    (WgslUnaryOperator::Negate, WgslLiteral::Float(f)) => {
                        Ok(WgslLiteral::Float(-f))
                    }
                    (WgslUnaryOperator::Not, WgslLiteral::Bool(v)) => Ok(WgslLiteral::Bool(!v)),
                    (WgslUnaryOperator::BitwiseNot, WgslLiteral::Integer(i)) => {
                        Ok(WgslLiteral::Integer(!i))
                    }
                    _ => Err(WgslError::InvalidExpression),
                }
            }
            WgslExpression::Comparison {
                left,
                comparison,
                right,
            } => {
                let left = left.evaluate(state)?;

                match comparison {
                    WgslComparison::Equal => Ok(WgslLiteral::Bool(left == right.evaluate(state)?)),
                    WgslComparison::NotEqual => {
                        Ok(WgslLiteral::Bool(left != right.evaluate(state)?))
                    }
                    WgslComparison::LessThan => {
                        Ok(WgslLiteral::Bool(left < right.evaluate(state)?))
                    }
                    WgslComparison::LessThanOrEqual => {
                        Ok(WgslLiteral::Bool(left <= right.evaluate(state)?))
                    }
                    WgslComparison::GreaterThan => {
                        Ok(WgslLiteral::Bool(left > right.evaluate(state)?))
                    }
                    WgslComparison::GreaterThanOrEqual => {
                        Ok(WgslLiteral::Bool(left >= right.evaluate(state)?))
                    }
                    WgslComparison::And => match left {
                        WgslLiteral::Bool(true) => right.evaluate(state),
                        f @ WgslLiteral::Bool(false) => Ok(f),
                        _ => Err(WgslError::InvalidExpression),
                    },
                    WgslComparison::Or => match left {
                        WgslLiteral::Bool(false) => right.evaluate(state),
                        f @ WgslLiteral::Bool(true) => Ok(f),
                        _ => Err(WgslError::InvalidExpression),
                    },
                }
            }
            WgslExpression::Parenthesized(e) => e.evaluate(state),
        }
    }

    fn reorder(&mut self) {
        return;
        // TODO

        // match self {
        //     WgslExpression::Operator {
        //         left,
        //         operator,
        //         right,
        //     } => {
        //         left.reorder();
        //         right.reorder();

        //         let self_priority = operator.priority();

        //         let left_priority = if let Self::Operator { ref operator, .. } = left.as_ref() {
        //             operator.priority()
        //         } else {
        //             0
        //         };

        //         let right_priority = if let Self::Operator { ref operator, .. } = right.as_ref() {
        //             operator.priority()
        //         } else {
        //             0
        //         };

        //         if left_priority < self_priority && left_priority < right_priority {
        //         } else if right_priority < self_priority && right_priority < left_priority {
        //         }
        //     }
        //     WgslExpression::Unary { operator, right } => todo!(),
        //     WgslExpression::Comparison {
        //         left,
        //         comparison,
        //         right,
        //     } => todo!(),
        //     WgslExpression::Parenthesized(_) => todo!(),
        //     _ => (),
        // }
    }

    fn from_chars<I: Iterator<Item = char> + Clone>(
        chars: &mut I,
        shallow: bool,
    ) -> Result<Option<Self>, WgslError> {
        let single = match chars.clone().next() {
            Some('!') => {
                chars.next().unwrap();

                Self::Unary {
                    operator: WgslUnaryOperator::Not,
                    right: Box::new(Self::from_chars(chars, true)?.ok_or(WgslError::NoExpression)?),
                }
            }
            Some('~') => {
                chars.next().unwrap();

                Self::Unary {
                    operator: WgslUnaryOperator::BitwiseNot,
                    right: Box::new(Self::from_chars(chars, true)?.ok_or(WgslError::NoExpression)?),
                }
            }
            Some('-') => {
                chars.next().unwrap();

                Self::Unary {
                    operator: WgslUnaryOperator::Negate,
                    right: Box::new(Self::from_chars(chars, true)?.ok_or(WgslError::NoExpression)?),
                }
            }
            Some('(') => {
                chars.next().unwrap();

                let expr =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                if !chars.next().is_some_and(|c| c == ')') {
                    Err(WgslError::NoClosingParenthesis)?;
                }

                Self::Parenthesized(expr)
            }
            Some(ch) if ch.is_numeric() => {
                let mut period = false; // The dot in floats
                let mut buffer = String::new();
                buffer.push(chars.next().unwrap());

                let mut radix = 10;
                let mut buffer_slice_start = 0;

                while let Some(ch) = chars.clone().next() {
                    match ch.to_ascii_lowercase() {
                        ch if ch.is_numeric() => {
                            buffer.push(chars.next().unwrap());
                        }
                        '_' => {
                            chars.next().unwrap();
                        }
                        '.' => {
                            buffer.push(chars.next().unwrap());

                            if period {
                                Err(WgslError::DuplicatePeriod)?;
                            }

                            period = true;
                        }
                        'b' => {
                            if buffer == "0" {
                                buffer.push(chars.next().unwrap());
                                buffer_slice_start = 2;
                                radix = 2;
                            } else {
                                Err(WgslError::InvalidBase)?;
                            }
                        }
                        'o' => {
                            if buffer == "0" {
                                buffer.push(chars.next().unwrap());
                                buffer_slice_start = 2;
                                radix = 8;
                            } else {
                                Err(WgslError::InvalidBase)?;
                            }
                        }
                        'x' => {
                            if buffer == "0" {
                                buffer.push(chars.next().unwrap());
                                buffer_slice_start = 2;
                                radix = 16;
                            } else {
                                Err(WgslError::InvalidBase)?;
                            }
                        }
                        _ => break,
                    }
                }

                if period {
                    Self::Literal(WgslLiteral::Float(
                        buffer[buffer_slice_start..]
                            .parse()
                            .map_err(WgslError::ParseFloatError)?,
                    ))
                } else {
                    Self::Literal(WgslLiteral::Integer(
                        i64::from_str_radix(&buffer[buffer_slice_start..], radix)
                            .map_err(WgslError::ParseIntError)?,
                    ))
                }
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let mut buffer = String::new();
                buffer.push(chars.next().unwrap());

                while let Some(ch) = chars.clone().next() {
                    if ch.is_alphanumeric() || ch == '_' {
                        buffer.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if buffer == "true" {
                    Self::Literal(WgslLiteral::Bool(true))
                } else if buffer == "false" {
                    Self::Literal(WgslLiteral::Bool(false))
                } else {
                    Self::Reference(buffer)
                }
            }
            _ => return Ok(None),
        };

        if shallow {
            return Ok(Some(single));
        }

        match chars.clone().next() {
            Some('+') => {
                chars.next().unwrap();

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Operator {
                    left,
                    operator: WgslOperator::Add,
                    right,
                }))
            }
            Some('-') => {
                chars.next().unwrap();

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Operator {
                    left,
                    operator: WgslOperator::Subtract,
                    right,
                }))
            }
            Some('*') => {
                chars.next().unwrap();

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Operator {
                    left,
                    operator: WgslOperator::Multiply,
                    right,
                }))
            }
            Some('/') => {
                chars.next().unwrap();

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Operator {
                    left,
                    operator: WgslOperator::Divide,
                    right,
                }))
            }
            Some('&') => {
                chars.next().unwrap();

                let left = Box::new(single);

                if matches!(chars.clone().next(), Some('&')) {
                    chars.next().unwrap();

                    let right =
                        Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                    return Ok(Some(WgslExpression::Comparison {
                        left,
                        comparison: WgslComparison::And,
                        right,
                    }));
                }

                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Operator {
                    left,
                    operator: WgslOperator::BitwiseAnd,
                    right,
                }))
            }
            Some('|') => {
                chars.next().unwrap();

                let left = Box::new(single);

                if matches!(chars.clone().next(), Some('|')) {
                    chars.next().unwrap();

                    let right =
                        Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                    return Ok(Some(WgslExpression::Comparison {
                        left,
                        comparison: WgslComparison::Or,
                        right,
                    }));
                }

                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Operator {
                    left,
                    operator: WgslOperator::BitwiseOr,
                    right,
                }))
            }
            Some('>') => {
                chars.next().unwrap();

                let mut comparison = WgslComparison::GreaterThan;

                if matches!(chars.clone().next(), Some('=')) {
                    comparison = WgslComparison::GreaterThanOrEqual;
                    chars.next().unwrap();
                }

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Comparison {
                    left,
                    comparison,
                    right,
                }))
            }
            Some('<') => {
                chars.next().unwrap();

                let mut comparison = WgslComparison::LessThan;

                if matches!(chars.clone().next(), Some('=')) {
                    comparison = WgslComparison::LessThanOrEqual;
                    chars.next().unwrap();
                }

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Comparison {
                    left,
                    comparison,
                    right,
                }))
            }
            Some('!') => {
                if matches!(chars.clone().nth(1), Some('=')) {
                    chars.next().unwrap();
                    chars.next().unwrap();
                } else {
                    return Ok(Some(single));
                }

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Comparison {
                    left,
                    comparison: WgslComparison::NotEqual,
                    right,
                }))
            }
            Some('=') => {
                if matches!(chars.clone().nth(1), Some('=')) {
                    chars.next().unwrap();
                    chars.next().unwrap();
                } else {
                    return Ok(Some(single));
                }

                let left = Box::new(single);
                let right =
                    Box::new(Self::from_chars(chars, false)?.ok_or(WgslError::NoExpression)?);

                Ok(Some(WgslExpression::Comparison {
                    left,
                    comparison: WgslComparison::Equal,
                    right,
                }))
            }
            _ => Ok(Some(single)),
        }
    }
}
