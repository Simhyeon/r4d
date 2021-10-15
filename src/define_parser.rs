use crate::utils::Utils;

/// Struct for deinition parsing
pub(crate) struct DefineParser{
    arg_cursor :DefineCursor,
    name: String,
    args: String,
    body: String,
    bind: bool,
    container: String,
}

impl DefineParser {
    pub fn new() -> Self {
        Self {
            arg_cursor : DefineCursor::Name,
            name : String::new(),
            args : String::new(),
            body : String::new(),
            bind : false,
            container : String::new(),
        }
    }

    /// Clear state
    fn clear(&mut self) {
        self.arg_cursor = DefineCursor::Name;
        self.name.clear();
        self.args.clear();
        self.body.clear();
        self.bind = false;
        self.container.clear();
    }

    /// Parse macro definition body
    ///
    /// NOTE: This method expects valid form of macro invocation
    /// which means given value should be presented without outer prentheses
    /// e.g. ) name,a1 a2=body text
    ///
    /// If definition doesn't comply with naming rules or syntaxes, if returnes "None"
    pub(crate) fn parse_define(&mut self, text: &str) -> Option<(String, String, String)> {
        self.clear(); // Start in fresh state
        let mut char_iter = text.chars().peekable();
        while let Some(ch) = char_iter.next() {
            match self.arg_cursor {
                DefineCursor::Name => {
                    if let ParseIgnore::Ignore = self.branch_name(ch) {continue;}
                    // If not valid name return None
                    if !self.is_valid_name(ch) { return None; }
                }
                DefineCursor::Args => {
                    if let ParseIgnore::Ignore = self.branch_args(ch) {continue;}
                    // If not valid name return None
                    if !self.is_valid_name(ch) { return None; }
                }
                // Add everything
                DefineCursor::Body => ()
            } 
            self.container.push(ch);
        }

        // This means pattern such as
        // $define(test,Test) 
        // -> This is not a valid pattern
        if self.args.len() == 0 && !self.bind {
            return None;
        }

        // End of body
        self.body.push_str(&self.container);

        Some((self.name.clone(), self.args.clone(), self.body.clone()))
    }

    /// Check if name complies with naming rule
    fn is_valid_name(&mut self, ch : char) -> bool {
        if self.container.len() == 0 { // Start of string
            // Not alphabetic 
            // $define( 1name ) -> Not valid
            if !ch.is_alphabetic() {
                return false;
            }
        } else { // middle of string
            // Not alphanumeric and not underscore
            // $define( na*1me ) -> Not valid
            // $define( na_1me ) -> Valid
            if !ch.is_alphanumeric() && ch != '_' {
                return false;
            }
        }
        true
    }
    
    // ---------
    // Start of branche methods
    // <DEF_BRANCH>
    
    fn branch_name(&mut self, ch: char) -> ParseIgnore {
        // $define(variable=something)
        // Don't set argument but directly bind variable to body
        if ch == '=' {
            self.name.push_str(&self.container);
            self.container.clear();
            self.arg_cursor = DefineCursor::Body;
            self.bind = true;
            ParseIgnore::Ignore
        } 
        else if Utils::is_blank_char(ch) {
            // This means pattern like this
            // $define( name ) -> name is registered
            // $define( na me ) -> na is ignored and take me instead
            if self.name.len() != 0 {
                self.container.clear();
                ParseIgnore::None
            } else {
                // Ignore
                ParseIgnore::Ignore
            }
        } 
        // Comma go to args
        else if ch == ',' {
            self.name.push_str(&self.container);
            self.container.clear();
            self.arg_cursor = DefineCursor::Args;
            ParseIgnore::Ignore
        } else {
            ParseIgnore::None
        }
    }

    fn branch_args(&mut self, ch: char) -> ParseIgnore {
        // Blank space separates arguments 
        // TODO: Why check name's length? Is it necessary?
        if Utils::is_blank_char(ch) && self.name.len() != 0 {
            if self.container.len() != 0 {
                self.args.push_str(&self.container);
                self.args.push(' ');
                self.container.clear();
            }
            ParseIgnore::Ignore
        } 
        // Go to body
        else if ch == '=' {
            self.args.push_str(&self.container);
            self.container.clear();
            self.arg_cursor = DefineCursor::Body; 
            ParseIgnore::Ignore
        } 
        // Others
        else {
            ParseIgnore::None
        }
    }

    // End of branche methods
    // </DEF_BRANCH>
    // ---------
}

pub(crate) enum DefineCursor {
    Name,
    Args,
    Body,
}

enum ParseIgnore {
    Ignore,
    None
}
