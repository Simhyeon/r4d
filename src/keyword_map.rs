use std::array::IntoIter;
use std::iter::FromIterator;
use std::collections::HashMap;
use crate::{AuthType, RadError};
use crate::utils::Utils;
use crate::models::RadResult;
use crate::arg_parser::ArgParser;
use crate::Processor;

type KMacroType = fn(&str, usize,&mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
pub struct KeywordMacroMap {
    pub(crate) macros : HashMap<String, KMacroSign>,
}

impl KeywordMacroMap {
    /// Creates empty map
    pub fn empty() -> Self {
        Self {
            macros: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        let map = HashMap::from_iter(IntoIter::new([
            ("bind".to_owned(),    KMacroSign::new("bind",    ["a_macro_name","a_value"],KeywordMacroMap::bind_depre)),
            ("declare".to_owned(), KMacroSign::new("declare", ["a_macro_names"],KeywordMacroMap::declare)),
            ("fassert".to_owned(), KMacroSign::new("fassert", ["a_lvalue","a_rvalue"],KeywordMacroMap::assert_fail)),
            ("foreach".to_owned(), KMacroSign::new("foreach", ["a_array","a_body"],KeywordMacroMap::foreach)),
            ("forline".to_owned(), KMacroSign::new("forline", ["a_iterable","a_body"],KeywordMacroMap::forline)),
            ("forloop".to_owned(), KMacroSign::new("forloop", ["a_min","a_max","a_body"],KeywordMacroMap::forloop)),
            ("global".to_owned(),  KMacroSign::new("global",  ["a_macro_name","a_value"],KeywordMacroMap::global_depre)),
            ("if".to_owned(),      KMacroSign::new("if",      ["a_boolean","a_if_expr"],KeywordMacroMap::if_cond)),
            ("ifelse".to_owned(),  KMacroSign::new("ifelse",  ["a_boolean","a_if_expr","a_else_expr"],KeywordMacroMap::ifelse)),
            ("ifdef".to_owned(),   KMacroSign::new("ifdef",   ["a_macro_name","a_if_expr"],KeywordMacroMap::ifdef)),
            ("ifdefel".to_owned(), KMacroSign::new("ifdefel", ["a_macro_name","a_if_expr","a_else_expr"],KeywordMacroMap::ifdefel)),
            ("ifenv".to_owned(),   KMacroSign::new("ifenv",   ["a_env_name","a_if_expr"],KeywordMacroMap::ifenv)),
            ("ifenvel".to_owned(), KMacroSign::new("ifenvel", ["a_env_name","a_if_expr","a_else_expr"],KeywordMacroMap::ifenvel)),
            ("let".to_owned(),     KMacroSign::new("let",     ["a_macro_name","a_value"],KeywordMacroMap::bind_to_local)),
            ("pause".to_owned(),   KMacroSign::new("pause",   ["a_pause?"],KeywordMacroMap::pause)),
            ("repl".to_owned(),    KMacroSign::new("repl",    ["a_macro_name","a_new_value"],KeywordMacroMap::replace)),
            ("static".to_owned(),  KMacroSign::new("static",  ["a_macro_name","a_value"],KeywordMacroMap::define_static)),
            ("sep".to_owned(),     KMacroSign::new("sep",     ["separator","a_array"],KeywordMacroMap::separate_array)),
        ]));
        Self {
            macros: map,
        }
    }

    /// Get Function pointer from map
    pub fn get_keyword_macro(&self, name: &str) -> Option<&KMacroType> {
        if let Some(mac) = self.macros.get(name) {
            Some(&mac.logic)
        } else {
            None
        }
    }

    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    // ----------
    // Keyword Macros start

    /// Pause every macro expansion
    ///
    /// Only other pause call is evaluated
    ///
    /// # Usage
    /// 
    /// $pause(true)
    /// $pause(false)
    fn pause(args: &str, level: usize,processor : &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, true) {
            let arg = &processor.parse_chunk_args(level, "", &args[0])?;

            if let Ok(value) =Utils::is_arg_true(arg) {
                if value {
                    processor.state.paused = true;
                } else {
                    processor.state.paused = false;
                }
                Ok(None)
            } 
            // Failed to evaluate
            else {
                Err(RadError::InvalidArgument(format!("Pause requires either true/false or zero/nonzero integer, but given \"{}\"", arg)))
            }
        } else {
            Err(RadError::InvalidArgument("Pause requires an argument".to_owned()))
        }
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage 
    ///
    /// $foreach(\*a,b,c*\,$:)
    fn foreach(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let mut sums = String::new();
            let loopable = &processor.parse_chunk_args(level, "", &args[0])?;

            for value in loopable.split(',') {
                let result = processor.parse_chunk_args(level, "", &args[1].replace("$:", value))?;
                sums.push_str(&result);
            }
            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument".to_owned()))
        }
    }

    /// Loop around given values split by new line and substitute iterators  with the value
    ///
    /// # Usage 
    ///
    /// $forline(TTT,$:)
    fn forline(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let mut sums = String::new();
            let loopable = &processor.parse_chunk_args(level, "", &args[0])?;

            for value in loopable.lines() {
                let result = processor.parse_chunk_args(level, "", &args[1].replace("$:", value))?;
                sums.push_str(&result);
            }
            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument("Forline requires two argument".to_owned()))
        }
    }

    /// For loop around given min, max value and finally substitue iterators with value
    ///
    /// # Usage
    ///
    /// $forloop(1,5,$:)
    fn forloop(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, true) {
            let mut sums = String::new();

            let min_src = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;
            let max_src = processor.parse_chunk_args(level, "",&Utils::trim(&args[1]))?;

            let min: usize; 
            let max: usize; 
            if let Ok(num) = min_src.parse::<usize>() {
                min = num;
            } else { 
                return Err(RadError::InvalidArgument(format!("Forloop's min value should be non zero positive integer but given {}", min_src))); 
            }
            if let Ok(num) = max_src.parse::<usize>() {
                max = num
            } else { 
                return Err(RadError::InvalidArgument(format!("Forloop's max value should be non zero positive integer but given \"{}\"", max_src))); 
            }
            
            for value in min..=max {
                let result = processor.parse_chunk_args(level, "", &args[2].replace("$:", &value.to_string()))?;
                sums.push_str(&result);
            }

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument("Forloop requires two argument".to_owned()))
        }
    }

    /// Print content according to given condition
    ///
    /// # Usage 
    ///
    /// $if(evaluation, ifstate)
    fn if_cond(args: &str,level:usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let boolean = &processor.parse_chunk_args(level,"",&args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(&Utils::trim(boolean));
            if let Ok(cond) = cond {
                if cond { 
                    let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                    return Ok(Some(if_expr)); 
                }
            } else {
                return Err(RadError::InvalidArgument(format!("If requires either true/false or zero/nonzero integer but given \"{}\"", boolean)))
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument("if requires two arguments".to_owned()))
        }
    }

    /// Print content according to given condition
    ///
    /// # Usage 
    ///
    /// $ifelse(evaluation, \*ifstate*\, \*elsestate*\)
    fn ifelse(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, true) {
            let boolean = &processor.parse_chunk_args(level,"",&args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(&Utils::trim(boolean));
            if let Ok(cond) = cond {
                if cond { 
                    let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                    return Ok(Some(if_expr)); 
                }
            } else {
                return Err(RadError::InvalidArgument(format!("Ifelse requires either true/false or zero/nonzero integer but given \"{}\"", boolean)))
            }

            // Else state
            let else_expr = processor.parse_chunk_args(level, "", &args[2])?;
            return Ok(Some(else_expr));
        } else {
            Err(RadError::InvalidArgument("ifelse requires three argument".to_owned()))
        }
    }

    /// If macro exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifdef(macro_name, expr)
    fn ifdef(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;
            let map = processor.get_map();

            let boolean = map.contains_any_macro(&name);
            // Return true or false by the definition
            if boolean { 
                let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                return Ok(Some(if_expr)); 
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("ifdef requires two arguments".to_owned()))
        }
    }

    /// If macro exists, then execute expresion else exectue another
    ///
    /// # Usage
    ///
    /// $ifdefelse(macro_name,expr,expr2)
    fn ifdefel(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;
            let map = processor.get_map();

            let boolean = map.contains_any_macro(&name);
            // Return true or false by the definition
            if boolean { 
                let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                return Ok(Some(if_expr)); 
            } else {
                let else_expr = processor.parse_chunk_args(level,"",&args[2])?;
                return Ok(Some(else_expr)); 
            }
        } else {
            Err(RadError::InvalidArgument("ifdefel requires three arguments".to_owned()))
        }
    }

    /// If env exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifenv(env_name, expr)
    fn ifenv(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenv", AuthType::ENV,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;

            let boolean = if let Ok(_) = std::env::var(name) {
                true
            } else {
                false
            };

            // Return true or false by the definition
            if boolean { 
                let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                return Ok(Some(if_expr)); 
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("ifenv requires two arguments".to_owned()))
        }
    }

    /// If env exists, then execute expresion else execute another
    ///
    /// # Usage
    ///
    /// $ifenvel(env_name,expr,expr2)
    fn ifenvel(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenvel", AuthType::ENV,processor)? {
            return Ok(None);
        }
        if let Some(args) = ArgParser::new().args_with_len(args, 3, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;

            let boolean = if let Ok(_) = std::env::var(name) {
                true
            } else {
                false
            };

            // Return true or false by the definition
            if boolean { 
                let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                return Ok(Some(if_expr)); 
            } else {
                let else_expr = processor.parse_chunk_args(level,"",&args[2])?;
                return Ok(Some(else_expr)); 
            }
        } else {
            Err(RadError::InvalidArgument("ifenvel requires three arguments".to_owned()))
        }
    }

    /// Replace value
    ///
    /// # Usage
    ///
    /// $repl(macro,value)
    fn replace(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;
            let target = args[1].as_str();
            if !processor.get_map().replace(&name, target) {
                return Err(RadError::InvalidArgument(format!("{} doesn't exist, thus cannot replace it's content", name)))
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Replace requires two arguments".to_owned()))
        }
    }

    /// Assert fail
    ///
    /// # Usage
    ///
    /// $fassert(abc,abc)
    fn assert_fail(args: &str, level: usize,processor: &mut Processor) -> RadResult<Option<String>> {
        let result = processor.parse_chunk_args(level, "", args);
        if let Err(_) = result {
            processor.track_assertion(true)?;
            Ok(None)
        } else {
            processor.track_assertion(false)?;
            Err(RadError::AssertFail)
        }
    }

    #[deprecated(since = "1.2", note = "Bind is deprecated and will be removed in 2.0")]
    fn bind_depre(args: &str, level:usize, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.log_warning("Bind is deprecated and will be removed in 2.0 version. Use let instead.")?;
        Self::bind_to_local(args, level, processor)
    }

    /// Declare an empty macros
    ///
    /// # Usage
    ///
    /// $declare(n1,n2,n3)
    fn declare(args: &str, level:usize, processor: &mut Processor) -> RadResult<Option<String>> {
        let names = processor.parse_chunk_args(level, "",&Utils::trim(args))?;
        // TODO Create empty macro rules
        let custom_rules = names
            .split(',')
            .map(|name| { (Utils::trim(name),"","") } )
            .collect::<Vec<(String,&str,&str)>>();

        // Check overriding. Warn or yield error
        for (name,_,_) in custom_rules.iter() {
            if processor.get_map().contains_any_macro(&name) {
                if processor.state.strict {
                    return Err(RadError::InvalidMacroName(format!("Declaring a macro with a name already existing : \"{}\"", name)))
                } else {
                    processor.log_warning(&format!("Declaring a macro with a name already existing : \"{}\"", name))?;
                }
            }
        }

        // Add custom rules
        processor.add_custom_rules(custom_rules)?;
        Ok(None)
    }

    /// Declare a local macro
    ///
    /// Local macro gets deleted after macro execution
    ///
    /// # Usage
    ///
    /// $let(name,value)
    fn bind_to_local(args: &str, level:usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;
            let value = processor.parse_chunk_args(level, "",&Utils::trim(&args[1]))?;
            // Let shadows varaible so it is ok to have existing name
            // TODO
            // I'm not so sure if Level 1 is fine for all cases?
            processor.get_map().new_local(1, &name, &value);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Let requires two argument".to_owned()))
        }
    }

    /// Global macro (Deprecated)
    ///
    /// This is technically same with static
    /// This macro will be completely removed in 2.0
    #[deprecated(since = "1.2", note = "Global is deprecated and will be removed in 2.0")]
    fn global_depre(args: &str, level: usize, processor: &mut Processor) -> RadResult<Option<String>> {
        processor.log_warning("Global is deprecated and will be removed in 2.0 version. Use static instead.")?;
        Self::define_static(args,level,processor)
    }

    /// Define a static macro
    ///
    /// # Usage
    ///
    /// $static(name,value)
    fn define_static(args: &str, level : usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let name = processor.parse_chunk_args(level, "",&Utils::trim(&args[0]))?;
            let value = processor.parse_chunk_args(level, "",&Utils::trim(&args[1]))?;
            // Macro name already exists
            if processor.get_map().contains_any_macro(&name) {
                // Strict mode prevents overriding
                // Return error
                if processor.state.strict {
                    return Err(RadError::InvalidMacroName(format!("Creating a static macro with a name already existing : \"{}\"", name)));
                } else {
                    // Its warn-able anyway
                    processor.log_warning(&format!("Creating a static macro with a name already existing : \"{}\"", name))?;
                }
            }
            processor.add_static_rules(vec![(&name,&value)])?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Static requires two argument".to_owned()))
        }
    }

    /// Separate an array
    ///
    /// # Usage
    ///
    /// $sep( ,1,2,3,4,5)
    fn separate_array(args: &str, level : usize, processor: &mut Processor) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let separator = processor.parse_chunk_args(level, "",&args[0])?;
            let array = processor.parse_chunk_args(level, "",&Utils::trim(&args[1]))?;
            let mut array = array.split(',').into_iter();
            let mut splited = String::new();

            if let Some(first) = array.next() {
                splited.push_str(first);

                for item in array {
                    splited.push_str(&format!("{}{}",separator,item));
                }
            }

            Ok(Some(splited))
        } else {
            Err(RadError::InvalidArgument("sep requires two argument".to_owned()))
        }
    }


    // Keyword macros end
    // ----------
}

/// Keyword Macro signature
#[derive(Clone)]
pub(crate) struct KMacroSign {
    name: String,
    args: Vec<String>,
    pub logic: KMacroType,
}

impl KMacroSign {
    pub fn new(name: &str, args: impl IntoIterator<Item = impl AsRef<str>>, logic: KMacroType) -> Self {
        let args = args.into_iter().map(|s| s.as_ref().to_owned()).collect::<Vec<String>>();
        Self {
            name : name.to_owned(),
            args,
            logic,
        }
    }
}

impl std::fmt::Display for KMacroSign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut inner = self.args.iter().fold(String::new(),|acc, arg| acc + &arg + ",");
        // This removes last "," character
        inner.pop();
        write!(f,"${}({})", self.name, inner)
    }
}

#[cfg(feature = "signature")]
impl From<&KMacroSign> for crate::sigmap::MacroSignature {
    fn from(bm: &KMacroSign) -> Self {
        Self {
            variant: crate::sigmap::MacroVariant::Keyword,
            name: bm.name.to_owned(),
            args: bm.args.to_owned(),
            expr: bm.to_string(),
        }
    }
}
