//! Stable, user-facing `CorexPM` diagnostics.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Broad diagnostic families. Codes are part of the public support contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorFamily {
    /// Registry communication or metadata errors.
    Registry,
    /// Dependency resolution errors.
    Resolve,
    /// Content-addressed store errors.
    Store,
    /// Lockfile parsing or validation errors.
    Lockfile,
    /// Security policy violations.
    Security,
    /// Lifecycle or package script errors.
    Script,
    /// Workspace graph errors.
    Workspace,
    /// Command-line usage errors.
    Cli,
}

impl ErrorFamily {
    /// Returns the stable prefix allocated to this family.
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Registry => "CXREG",
            Self::Resolve => "CXRESOLVE",
            Self::Store => "CXSTORE",
            Self::Lockfile => "CXLOCK",
            Self::Security => "CXSEC",
            Self::Script => "CXSCRIPT",
            Self::Workspace => "CXWORK",
            Self::Cli => "CXCLI",
        }
    }
}

/// A compact diagnostic suitable for CLI output and structured serialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    family: ErrorFamily,
    number: u16,
    message: String,
    help: Option<String>,
}

impl Diagnostic {
    /// Creates a diagnostic with a stable numeric code within its family.
    #[must_use]
    pub fn new(family: ErrorFamily, number: u16, message: impl Into<String>) -> Self {
        Self {
            family,
            number,
            message: message.into(),
            help: None,
        }
    }

    /// Adds actionable help text.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Returns a stable code such as `CXCLI0001`.
    #[must_use]
    pub fn code(&self) -> String {
        format!("{}{:04}", self.family.prefix(), self.number)
    }
}

impl Display for Diagnostic {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.message)?;
        if let Some(help) = &self.help {
            write!(formatter, "\nhelp: {help}")?;
        }
        Ok(())
    }
}

impl Error for Diagnostic {}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, ErrorFamily};

    #[test]
    fn diagnostic_codes_are_zero_padded() {
        let error = Diagnostic::new(ErrorFamily::Cli, 1, "not implemented");
        assert_eq!(error.code(), "CXCLI0001");
    }
}
