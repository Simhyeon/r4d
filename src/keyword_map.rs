use std::array::IntoIter;
use std::iter::FromIterator;
use std::collections::HashMap;
use crate::{AuthType, RadError};
use crate::utils::Utils;
use crate::models::RadResult;
use crate::arg_parser::ArgParser;
use crate::Processor;

type KeywordMacType = fn(&str, usize,&mut Processor) -> RadResult<Option<String>>;

#[derive(Clone)]
pub struct KeywordMacro {
    macros : HashMap<String, KeywordMacType>,
}

impl KeywordMacro {
    pub fn new() -> Self {
        let map = HashMap::from_iter(IntoIter::new([
            ("bind".to_owned(),    KeywordMacro::bind_depre       as KeywordMacType),
            ("declare".to_owned(), KeywordMacro::declare          as KeywordMacType),
            ("fassert".to_owned(), KeywordMacro::assert_fail      as KeywordMacType),
            ("foreach".to_owned(), KeywordMacro::foreach          as KeywordMacType),
            ("forloop".to_owned(), KeywordMacro::forloop          as KeywordMacType),
            ("global".to_owned(),  KeywordMacro::global_depre     as KeywordMacType),
            ("if".to_owned(),      KeywordMacro::if_cond          as KeywordMacType),
            ("ifdef".to_owned(),   KeywordMacro::ifdef            as KeywordMacType),
            ("ifdefel".to_owned(), KeywordMacro::ifdefel          as KeywordMacType),
            ("ifelse".to_owned(),  KeywordMacro::ifelse           as KeywordMacType),
            ("ifenv".to_owned(),   KeywordMacro::ifenv            as KeywordMacType),
            ("ifenvel".to_owned(), KeywordMacro::ifenvel          as KeywordMacType),
            ("let".to_owned(),     KeywordMacro::bind_to_local    as KeywordMacType),
            ("pause".to_owned(),   KeywordMacro::pause            as KeywordMacType),
            ("repl".to_owned(),    KeywordMacro::replace          as KeywordMacType),
            ("static".to_owned(),  KeywordMacro::define_static    as KeywordMacType),
        ]));
        Self {
            macros: map,
        }
    }

    /// Get Function pointer from map
    pub fn get(&self, name: &str) -> Option<&KeywordMacType> {
        if let Some(mac) = self.macros.get(name) {
            Some(mac)
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
                return Err(RadError::InvalidArgument(format!("Forloop's min value should be non zero positive integer but given {}", &args[0]))); 
            }
            if let Ok(num) = max_src.parse::<usize>() {
                max = num
            } else { 
                return Err(RadError::InvalidArgument(format!("Forloop's min value should be non zero positive integer gut given \"{}\"", &args[1]))); 
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

            let boolean = map.contains(&name);
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

            let boolean = map.contains(&name);
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

        // Check overriding. Wanr or yield error
        for (name,_,_) in custom_rules.iter() {
            if processor.get_map().contains(&name) {
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
            if processor.get_map().contains(&name) {
                // Strict mode prevents overriding
                // Return error
                if processor.state.strict {
                    return Err(RadError::InvalidMacroName(format!("Creating a static macro with a name already existing : \"{}\"", name)));
                } else {
                    // Its warn-able anyway
                    processor.log_warning(&format!("Creating a static macro with a name already existing : \"{}\"", name))?;
                }
            }
            processor.add_custom_rules(vec![(&name,"",&value)])?;
            Ok(None)
        } else {
            Err(RadError::InvalidArgument("Static requires two argument".to_owned()))
        }
    }

    // Keyword macros end
    // ----------
}
