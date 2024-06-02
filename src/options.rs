#[derive(PartialEq, Debug, Clone, Copy)]
pub enum NumberingMode {
    /// Do not number liens
    None,
    /// Number nonempty lines
    NonEmpty,
    /// Number all lines
    All,
}

/// Options to format the output
pub struct Options {
    /// Setting to number lines
    pub number: NumberingMode,

    /// Display a `$` after the end of each line
    pub show_ends: bool,

    /// Suppress repeated empty output lines
    pub squeeze_blank: bool,

    /// Display TAB characters as `^I`
    pub show_tabs: bool,

    /// Use `^` and `M-` notation, except for LFD and TAB
    pub show_nonprinting: bool,
}

impl Options {
    /// Create a new `Options` struct with default values
    pub fn new() -> Self {
        Self {
            number: NumberingMode::None,
            show_ends: false,
            squeeze_blank: false,
            show_tabs: false,
            show_nonprinting: false,
        }
    }

    /// Update with the number option
    pub fn number(mut self, number: NumberingMode) -> Self {
        self.number = number;
        self
    }

    /// Update with the show_ends option
    pub fn show_ends(mut self, show_ends: bool) -> Self {
        self.show_ends = show_ends;
        self
    }

    /// Update with the squeeze_blank option
    pub fn squeeze_blank(mut self, squeeze_blank: bool) -> Self {
        self.squeeze_blank = squeeze_blank;
        self
    }

    /// Update with the show_tabs option
    pub fn show_tabs(mut self, show_tabs: bool) -> Self {
        self.show_tabs = show_tabs;
        self
    }

    /// Update with the show_nonprinting option
    pub fn show_nonprinting(mut self, show_nonprinting: bool) -> Self {
        self.show_nonprinting = show_nonprinting;
        self
    }
}

impl Options {
    pub(crate) fn tab(&self) -> &'static str {
        if self.show_tabs {
            "^I"
        } else {
            "\t"
        }
    }

    pub(crate) fn end_of_line(&self) -> &'static str {
        if self.show_ends {
            "$\n"
        } else {
            "\n"
        }
    }

    /// We can write fast if we can simply copy the contents of the file to
    /// stdout, without augmenting the output with e.g. line numbers.
    pub(crate) fn can_write_fast(&self) -> bool {
        !(self.show_tabs
            || self.show_nonprinting
            || self.show_ends
            || self.squeeze_blank
            || self.number != NumberingMode::None)
    }
}
