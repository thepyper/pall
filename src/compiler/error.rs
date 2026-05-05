use std::fmt;

// Phase 1 steps 1.3, 1.5: Enum variants for programmatic error matching.
// Fleshed out in subsequent steps.

#[derive(Debug, Clone, PartialEq)]
pub enum CompileErrorKind {
    DuplicateMachineId,
    UnreachableTransition,
    MissingStateReference,
    InvalidLink,
    InvalidTimerType,
    InvalidSignalExpr,
    ReservedVariableName,
}

impl fmt::Display for CompileErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileErrorKind::DuplicateMachineId => write!(f, "duplicate machine id"),
            CompileErrorKind::UnreachableTransition => write!(f, "unreachable transition"),
            CompileErrorKind::MissingStateReference => write!(f, "missing state reference"),
            CompileErrorKind::InvalidLink => write!(f, "invalid link"),
            CompileErrorKind::InvalidTimerType => write!(f, "invalid timer type"),
            CompileErrorKind::InvalidSignalExpr => write!(f, "invalid signal expression"),
            CompileErrorKind::ReservedVariableName => write!(f, "reserved variable name"),
        }
    }
}

#[derive(Debug)]
pub struct CompileError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub kind: CompileErrorKind,
}

impl CompileError {
    pub fn new(kind: CompileErrorKind, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
            kind,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for CompileError {}

#[derive(Debug, Clone, PartialEq)]
pub enum TickErrorKind {
    UnknownState,
}

impl fmt::Display for TickErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TickErrorKind::UnknownState => write!(f, "unknown state"),
        }
    }
}

#[derive(Debug)]
pub struct TickError {
    pub message: String,
    pub kind: TickErrorKind,
}

impl fmt::Display for TickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)
    }
}

impl std::error::Error for TickError {}
