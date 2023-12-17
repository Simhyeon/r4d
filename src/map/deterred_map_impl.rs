#[cfg(not(feature = "wasm"))]
use crate::auth::AuthType;
#[cfg(not(feature = "wasm"))]
use crate::common::ProcessInput;
#[cfg(not(feature = "wasm"))]
use crate::common::{ContainerType, FileTarget, FlowControl, RelayTarget};
use crate::common::{ErrorBehaviour, MacroType, RadResult, STREAM_CONTAINER, STREAM_MACRO_NAME};
use crate::consts::MACRO_SPECIAL_ANON;
use crate::deterred_map::DeterredMacroMap;
use crate::formatter::Formatter;
use crate::parser::GreedyState;
use crate::utils::Utils;
use crate::ArgParser;
use crate::{trim, Processor, RadError};
use std::fs::File;
use std::io::{BufRead, BufReader};
#[cfg(not(feature = "wasm"))]
use std::path::PathBuf;

impl DeterredMacroMap {
    // ----------
    // Keyword Macros start

    /// Anon
    ///
    /// # Usage
    ///
    /// $anon(a=$a())
    pub(crate) fn add_anonymous_macro(
        args: &str,
        _: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        p.add_anon_macro(args)?;
        Ok(Some(String::from(MACRO_SPECIAL_ANON)))
    }

    /// Append content to a macro
    ///
    /// This is deterred because it needs level for local macro indexing
    ///
    /// Runtime + local macros can be appended.
    ///
    /// # Usage
    ///
    /// $append(macro_name,Content,tailer)
    pub(crate) fn append(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        let args = ap.args_to_vec(args, ',', GreedyState::Never);
        ap.set_strip(true);
        if args.len() >= 2 {
            let name =
                processor.parse_and_strip(&mut ap, level, "append", trim!(&args[0]).as_ref())?;
            let target = processor.parse_and_strip(&mut ap, level, "append", &args[1])?;
            let mut trailer = None;

            if args.len() >= 3 {
                trailer = Some(processor.parse_and_strip(&mut ap, level, "append", &args[2])?);
            }

            if let Some(name) = processor.contains_local_macro(level, &name) {
                if let Some(tt) = trailer {
                    let body = processor.get_local_macro_body(&name)?;
                    if !body.ends_with(&tt) && !body.is_empty() {
                        processor.append_local_macro(&name, &format!("{}{}", tt, target));
                        return Ok(None);
                    }
                }
                processor.append_local_macro(&name, &target);
            } else if processor.contains_macro(&name, MacroType::Runtime) {
                // Append to runtime
                if let Some(tt) = trailer {
                    let body = processor.get_runtime_macro_body(&name)?;
                    if !body.ends_with(&tt) && !body.is_empty() {
                        processor.append_macro(&name, &format!("{}{}", tt, target));
                        return Ok(None);
                    }
                }
                processor.append_macro(&name, &target);
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Macro \"{}\" doesn't exist",
                    name
                )));
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Append at least requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply map on array
    ///
    /// # Usage
    ///
    /// $map(macro_name,array)
    pub(crate) fn map_array(
        args: &str,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_name = p.parse_and_strip(&mut ap, level, "mapl", &trim!(&args[0]))?;
            let src = p.parse_and_strip(&mut ap, level, "map", &args[1])?;
            let array = src.split(',');

            let mut acc = String::new();
            for item in array {
                acc.push_str(
                    &p.execute_macro(level, "map", &macro_name, item)?
                        .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "map requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply map on lines
    ///
    /// # Usage
    ///
    /// $mapl(macro_name,lines)
    pub(crate) fn map_lines(
        args: &str,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_name = p.parse_and_strip(&mut ap, level, "mapl", &trim!(&args[0]))?;
            let src = p.parse_and_strip(&mut ap, level, "mapl", &args[1])?;
            let lines = src.lines();

            let mut acc = String::new();
            for item in lines {
                acc.push_str(
                    &p.execute_macro(level, "mapl", &macro_name, item)?
                        .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "mapl requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply map on file lines
    ///
    /// # Usage
    ///
    /// $mapf(macro_name,file_name)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn map_file(
        args: &str,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("mapf", AuthType::FIN, p)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_name = p.parse_and_strip(&mut ap, level, "mapl", &trim!(&args[0]))?;
            let file = BufReader::new(std::fs::File::open(p.parse_and_strip(
                &mut ap,
                level,
                "mapf",
                &trim!(&args[1]),
            )?)?)
            .lines();

            let mut acc = String::new();
            for line in file {
                let line = line?;
                acc.push_str(
                    &p.execute_macro(level, "mapf", &macro_name, &line)?
                        .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "mapf requires two arguments".to_owned(),
            ))
        }
    }

    /// Apply maps on captured expressions
    ///
    /// # Usage
    ///
    /// $grepmap(type,expr,macro,text)
    pub(crate) fn grep_map(
        args: &str,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 4) {
            ap.set_strip(true);
            let grep_type = &args[0];
            let match_expr = &args[1];
            let macro_name = trim!(&args[2]);
            let source = &args[3];

            let bufread = match grep_type.to_lowercase().as_str() {
                #[cfg(not(feature = "wasm"))]
                "file" => {
                    if !Utils::is_granted("grepmap", AuthType::FIN, p)? {
                        return Ok(None);
                    }
                    true
                }
                "text" => false,
                _ => {
                    return Err(RadError::InvalidArgument(format!(
                        "{} is not a valid grep type",
                        grep_type
                    )))
                }
            };

            if bufread && !std::path::Path::new(source).exists() {
                return Err(RadError::InvalidArgument(format!(
                    "Cannot find a file \"{}\" ",
                    source
                )));
            }

            if match_expr.is_empty() {
                return Err(RadError::InvalidArgument(
                    "Regex expression cannot be an empty string".to_string(),
                ));
            }

            let mut res = String::new();

            // If this regex is not cloned, "capture" should collect captured string into a allocated
            // vector. Which is generaly worse for performance.
            let reg = p.try_get_or_insert_regex(match_expr)?.clone();

            if !bufread {
                for cap in reg.captures_iter(source) {
                    let captured = cap.get(0).map_or("", |m| m.as_str());
                    let expanded = p
                        .execute_macro(level, "grepmap", &macro_name, captured)?
                        .unwrap_or_default();
                    res.push_str(&expanded);
                }
            } else {
                let lines = BufReader::new(File::open(std::path::Path::new(source))?).lines();

                for line in lines {
                    let line = line?;
                    for cap in reg.captures_iter(&line) {
                        let captured = cap.get(0).map_or("", |m| m.as_str());
                        let expanded = p
                            .execute_macro(level, "grepmap", &macro_name, captured)?
                            .unwrap_or_default();
                        res.push_str(&expanded);
                    }
                }
            }

            Ok(Some(res))
        } else {
            Err(RadError::InvalidArgument(
                "grepamp requires four arguments".to_owned(),
            ))
        }
    }

    /// Loop around given values which is separated by given separator
    ///
    /// # Usage
    ///
    /// $forby($:(),-,a-b-c)
    pub(crate) fn forby(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);
            let mut sums = String::new();
            let body = &args[0];
            let sep = &processor.parse_and_strip(&mut ap, level, "forby", &args[1])?;
            let loopable = &processor.parse_and_strip(&mut ap, level, "forby", &args[2])?;
            for (count, value) in loopable.split_terminator(sep).enumerate() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = &processor.parse_and_strip(&mut ap, level, "forby", body)?;

                sums.push_str(result);
            }

            // Clear local macro
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Foreach requires two argument".to_owned(),
            ))
        }
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $foreach($:(),a,b,c)
    pub(crate) fn foreach(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let mut sums = String::new();
            let body = &args[0];
            let loop_src = processor.parse_and_strip(&mut ap, level, "foreach", &args[1])?;
            let loopable = trim!(&loop_src);
            for (count, value) in loopable.as_ref().split(',').enumerate() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = &processor.parse_and_strip(&mut ap, level, "foreach", body)?;

                sums.push_str(result);
            }

            // Clear local macro
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Foreach requires two argument".to_owned(),
            ))
        }
    }

    /// Loop around given values split by new line and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $forline($:(),Content)
    pub(crate) fn forline(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let mut sums = String::new();
            let body = &args[0];
            let loop_src = processor.parse_and_strip(&mut ap, level, "forline", &args[1])?;
            let loopable = trim!(&loop_src);
            let mut count = 1;
            for value in loopable.lines() {
                // This overrides value
                processor.add_new_local_macro(level, "a_LN", &count.to_string());
                processor.add_new_local_macro(level, ":", value);
                let result = processor.parse_and_strip(&mut ap, level, "forline", body)?;
                sums.push_str(&result);
                count += 1;
            }
            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Forline requires two argument".to_owned(),
            ))
        }
    }

    /// For loop around given min, max value and finally substitue iterators with value
    ///
    /// # Usage
    ///
    /// $forloop($:(),1,5)
    pub(crate) fn forloop(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);
            let mut sums = String::new();

            let body = &args[0];
            let min_src =
                trim!(&processor.parse_and_strip(&mut ap, level, "forloop", &args[1])?).to_string();
            let max_src =
                trim!(&processor.parse_and_strip(&mut ap, level, "forloop", &args[2])?).to_string();

            let min = if let Ok(num) = min_src.parse::<usize>() {
                num
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Forloop's min value should be non zero positive integer but given {}",
                    min_src
                )));
            };
            let max = if let Ok(num) = max_src.parse::<usize>() {
                num
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Forloop's max value should be non zero positive integer but given \"{}\"",
                    max_src
                )));
            };
            let mut result: String;
            for value in min..=max {
                processor.add_new_local_macro(level, ":", &value.to_string());
                result = processor.parse_and_strip(&mut ap, level, "forloop", body)?;

                sums.push_str(&result);
                result.clear();
            }

            // Clear local macro
            processor.remove_local_macro(level, ":");

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument(
                "Forloop requires two argument".to_owned(),
            ))
        }
    }

    /// Log macro information
    ///
    /// # Usage
    ///
    /// $logm(mac)
    pub(crate) fn log_macro_info(
        args: &str,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let macro_name = trim!(&p.parse_and_strip(&mut ap, level, "logm", args)?).to_string();
        let body = if let Some(name) = p.contains_local_macro(level, &macro_name) {
            p.get_local_macro_body(&name)?.to_string()
        } else if let Ok(body) = p.get_runtime_macro_body(&macro_name) {
            body.to_string()
        } else {
            return Err(RadError::InvalidArgument(format!(
                "Macro \"{}\" doesn't exist",
                &macro_name
            )));
        };
        p.log_message(&body)?;
        Ok(None)
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $if(evaluation, ifstate)
    pub(crate) fn if_cond(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let boolean = &processor.parse_and_strip(&mut ap, level, "if", &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(boolean);
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_and_strip(&mut ap, level, "if", &args[1])?;
                    return Ok(Some(if_expr));
                }
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "If requires either true/false or zero/nonzero integer but given \"{}\"",
                    boolean
                )));
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "if requires two arguments".to_owned(),
            ))
        }
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $ifelse(evaluation, \*ifstate*\, \*elsestate*\)
    pub(crate) fn ifelse(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let boolean = &processor.parse_and_strip(&mut ap, level, "ifelse", &args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(boolean);
            if let Ok(cond) = cond {
                if cond {
                    let if_expr = processor.parse_and_strip(&mut ap, level, "ifelse", &args[1])?;
                    return Ok(Some(if_expr));
                }
            } else {
                return Err(RadError::InvalidArgument(format!(
                    "Ifelse requires either true/false or zero/nonzero integer but given \"{}\"",
                    boolean
                )));
            }

            // Else state
            let else_expr = processor.parse_and_strip(&mut ap, level, "ifelse", &args[2])?;
            Ok(Some(else_expr))
        } else {
            Err(RadError::InvalidArgument(
                "ifelse requires three argument".to_owned(),
            ))
        }
    }

    /// If macro exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifdef(macro_name, expr)
    pub(crate) fn ifdef(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifdef", &args[0])?).to_string();

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifdef", &args[1])?;
                return Ok(Some(if_expr));
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "ifdef requires two arguments".to_owned(),
            ))
        }
    }

    /// If macro exists, then execute expresion else exectue another
    ///
    /// # Usage
    ///
    /// $ifdefel(macro_name,expr,expr2)
    pub(crate) fn ifdefel(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifdefel", &args[0])?).to_string();

            let boolean = processor.contains_macro(&name, MacroType::Any);
            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifdefel", &args[1])?;
                Ok(Some(if_expr))
            } else {
                let else_expr = processor.parse_and_strip(&mut ap, level, "ifdefel", &args[2])?;
                Ok(Some(else_expr))
            }
        } else {
            Err(RadError::InvalidArgument(
                "ifdefel requires three arguments".to_owned(),
            ))
        }
    }

    /// If env exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifenv(env_name, expr)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn ifenv(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenv", AuthType::ENV, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifenv", &args[0])?).to_string();

            let boolean = std::env::var(name).is_ok();

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifenv", &args[1])?;
                return Ok(Some(if_expr));
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "ifenv requires two arguments".to_owned(),
            ))
        }
    }

    /// If env exists, then execute expresion else execute another
    ///
    /// # Usage
    ///
    /// $ifenvel(env_name,expr,expr2)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn ifenvel(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenvel", AuthType::ENV, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 3) {
            ap.set_strip(true);

            let name =
                trim!(&processor.parse_and_strip(&mut ap, level, "ifenvel", &args[0])?).to_string();

            let boolean = std::env::var(name).is_ok();

            // Return true or false by the definition
            if boolean {
                let if_expr = processor.parse_and_strip(&mut ap, level, "ifenvel", &args[1])?;
                Ok(Some(if_expr))
            } else {
                let else_expr = processor.parse_and_strip(&mut ap, level, "ifenvel", &args[2])?;
                Ok(Some(else_expr))
            }
        } else {
            Err(RadError::InvalidArgument(
                "ifenvel requires three arguments".to_owned(),
            ))
        }
    }

    /// Expand expression
    ///
    /// This strip an expression and then expand it
    ///
    /// # Usage
    ///
    /// $expand(expression)
    pub(crate) fn expand_expression(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = ArgParser::new().strip(args);
        let result = processor.parse_chunk_args(level, "", &args)?;

        Ok(if result.is_empty() {
            None
        } else {
            Some(result)
        })
    }

    /// Assert fail
    ///
    /// This has to be deterred macro because it's value should be evaluated later
    ///
    /// # Usage
    ///
    /// $fassert(abc)
    pub(crate) fn assert_fail(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let backup = processor.state.behaviour;
        processor.state.behaviour = ErrorBehaviour::Assert;

        let mut ap = ArgParser::new().no_strip();
        let result = processor.parse_and_strip(&mut ap, level, "fassert", args);
        processor.state.behaviour = backup;
        if result.is_err() {
            processor.track_assertion(true)?;
            Ok(None)
        } else {
            processor.track_assertion(false)?;
            Err(RadError::AssertFail)
        }
    }

    /// Consume streaming
    ///
    /// # Usage
    ///
    /// $stream(macro_name)
    /// $consume()
    pub(crate) fn consume(_: &str, level: usize, p: &mut Processor) -> RadResult<Option<String>> {
        let macro_name = p.get_runtime_macro_body(STREAM_MACRO_NAME)?.to_owned();
        let content = p.get_runtime_macro_body(STREAM_CONTAINER)?.to_owned();

        // You should pop first because it has to be evaluated to open
        p.state.relay.pop();

        let result = p.execute_macro(level, "consume", &macro_name, &content)?;

        p.replace_macro(STREAM_MACRO_NAME, &String::default()); // Clean macro
        Ok(result)
    }

    /// Queue processing
    ///
    /// # Usage
    ///
    /// $que(Sentence to process)
    pub(crate) fn queue_content(
        args: &str,
        _: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        processor.insert_queue(args);
        Ok(None)
    }

    /// Queue processing
    ///
    /// # Usage
    ///
    /// $ifque(true,Sentence to process)
    pub(crate) fn if_queue_content(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let boolean = &processor.parse_and_strip(&mut ap, level, "ifque", &args[0])?;
            let cond = Utils::is_arg_true(boolean)?;
            if cond {
                processor.insert_queue(&args[1]);
            }
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "ifque requires two argument".to_owned(),
            ))
        }
    }

    pub(crate) fn escape_blanks(
        _: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if level != 1 {
            return Err(RadError::UnallowedMacroExecution(
                "\"EB\" is only available on first level".to_string(),
            ));
        }
        processor.state.lexor_escape_blanks = true;
        Ok(None)
    }

    /// Read to
    ///
    /// # Usage
    ///
    /// $readto(file_a,file_b)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn read_to(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        // Needs both permission
        if !Utils::is_granted("readto", AuthType::FIN, processor)?
            || !Utils::is_granted("readto", AuthType::FOUT, processor)?
        {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let file_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                "readto",
                trim!(&args[0]).as_ref(),
            )?);
            let to_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                "readto",
                trim!(&args[1]).as_ref(),
            )?);
            if file_path == to_path {
                return Err(RadError::InvalidArgument(format!(
                    "readto cannot read from and into a same file \"{}\"",
                    file_path.display()
                )));
            }
            let mut raw_include = false;
            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;
                Utils::check_file_sanity(processor, &canonic)?;

                // Check path sanity if to_path exists
                if to_path.exists() {
                    Utils::check_file_sanity(processor, &to_path.canonicalize()?)?;
                }
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 3 {
                    raw_include = Utils::is_arg_true(&processor.parse_and_strip(
                        &mut ap,
                        level,
                        "readto",
                        trim!(&args[2]).as_ref(),
                    )?)?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.flow_control = FlowControl::Escape;
                    }
                }

                let file_target = FileTarget::from_path(&to_path)?;
                processor.state.relay.push(RelayTarget::File(file_target));

                // Create chunk
                let chunk = processor.process_file_as_chunk(&file_path, ContainerType::Expand)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.flow_control = FlowControl::None; // Recover state
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                processor.state.relay.pop(); // Pop relay
                Ok(chunk)
            } else {
                Err(RadError::InvalidArgument(format!(
                    "readto cannot read non-file \"{}\"",
                    file_path.display()
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "readto requires two argument".to_owned(),
            ))
        }
    }

    /// Read in
    ///
    /// # Usage
    ///
    /// $readin(file_a)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn read_in(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        use crate::WarningType;

        if !Utils::is_granted("readin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 1) {
            ap.set_strip(true);
            let file_path = PathBuf::from(processor.parse_and_strip(
                &mut ap,
                level,
                "readin",
                trim!(&args[0]).as_ref(),
            )?);
            let mut raw_include = false;
            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;
                Utils::check_file_sanity(processor, &canonic)?;
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 2 {
                    raw_include = Utils::is_arg_true(&processor.parse_and_strip(
                        &mut ap,
                        level,
                        "readin",
                        trim!(&args[1]).as_ref(),
                    )?)?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.flow_control = FlowControl::Escape;
                    }
                }

                if let Some(relay) = processor.state.relay.last().clone() {
                    processor.log_warning(
                        &format!("Read file's content will be relayed to \"{:?}\"", relay),
                        WarningType::Sanity,
                    )?;
                }

                // Create chunk
                let chunk = processor.process_file(&file_path)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.flow_control = FlowControl::None;
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                Ok(chunk)
            } else {
                Err(RadError::InvalidArgument(format!(
                    "readin cannot read non-file \"{}\"",
                    file_path.display()
                )))
            }
        } else {
            Err(RadError::InvalidArgument(
                "readin requires an argument".to_owned(),
            ))
        }
    }

    /// Execute macro
    ///
    /// # Usage
    ///
    /// $exec(macro_name,macro_args)
    pub(crate) fn execute_macro(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let macro_name =
                trim!(&processor.parse_and_strip(&mut ap, level, "exec", &args[0])?).to_string();
            let args = processor.parse_and_strip(&mut ap, level, "exec", &args[1])?;
            let result = processor
                .execute_macro(level, "exec", &macro_name, &args)?
                .unwrap_or_default();
            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "exec requires two argument".to_owned(),
            ))
        }
    }

    /// Create multiple macro executions from given csv value
    ///
    /// # Usage
    ///
    /// $from(macro_name,\*1,2,3
    /// 4,5,6*\)
    ///
    /// $from+(macro_name,
    /// 1,2,3
    /// 4,5,6
    /// )
    pub(crate) fn spread_data(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);

            let expanded_name = &processor.parse_and_strip(&mut ap, level, "spread", &args[0])?;
            let expanded_data = &processor.parse_and_strip(&mut ap, level, "spread", &args[1])?;
            let macro_name = trim!(expanded_name);
            let macro_data = trim!(expanded_data);

            let result =
                Formatter::csv_to_macros(&macro_name, &macro_data, &processor.state.newline)?;

            // Disable debugging for nested macro expansion
            #[cfg(feature = "debug")]
            let original = processor.is_debug();

            // Now this might look strange,
            // "Why not just enclose two lines with curly brackets?"
            // The answer is such appraoch somehow doesn't work.
            // Compiler cannot deduce the variable original and will yield error on
            // processor.debug(original)
            #[cfg(feature = "debug")]
            processor.set_debug(false);

            // Parse macros
            let result = processor.parse_and_strip(&mut ap, level, "spread", &result)?;

            // Set custom prompt log to indicate user thatn from macro doesn't support
            // debugging inside macro expansion
            #[cfg(feature = "debug")]
            {
                use crate::debugger::DebugSwitch;
                processor.set_debug(original);
                match processor.get_debug_switch() {
                    DebugSwitch::StepMacro | DebugSwitch::NextMacro => {
                        processor.set_prompt("\"Spread macro\"")
                    }
                    _ => (),
                }
            }

            Ok(Some(result))
        } else {
            Err(RadError::InvalidArgument(
                "spread requires two arguments".to_owned(),
            ))
        }
    }

    /// Paste given file's content
    ///
    /// Every macros within the file is also expanded
    ///
    /// Include read file's content into a single string and print out.
    /// This enables ergonomic process of macro execution. If you want file
    /// inclusion to happen as bufstream, use read instead.
    ///
    /// # Usage
    ///
    /// $include(path)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn include(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("include", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        let args = ap.args_to_vec(args, ',', GreedyState::Never);
        ap.set_strip(true);
        if !args.is_empty() {
            let mut file_path = PathBuf::from(
                trim!(&processor.parse_and_strip(&mut ap, level, "include", &args[0])?).as_ref(),
            );
            let mut raw_include = false;

            // if current input is not stdin and file path is relative
            // Create new file path that starts from current file path
            if let ProcessInput::File(path) = &processor.state.current_input {
                if file_path.is_relative() {
                    // It is ok get parent because any path that has a length can return parent
                    file_path = path.parent().unwrap().join(file_path);
                }
            }

            if file_path.is_file() {
                let canonic = file_path.canonicalize()?;

                Utils::check_file_sanity(processor, &canonic)?;
                // Set sandbox after error checking or it will act starngely
                processor.set_sandbox(true);

                // Optionally enable raw mode
                if args.len() >= 2 {
                    raw_include = Utils::is_arg_true(
                        &processor.parse_and_strip(&mut ap, level, "include", &args[1])?,
                    )?;

                    // You don't have to backup pause state because include wouldn't be triggered
                    // at the first place, if paused was true
                    if raw_include {
                        processor.state.flow_control = FlowControl::Escape;
                    }
                }

                let container_type = if level != 1 {
                    ContainerType::Argument
                } else {
                    ContainerType::Expand
                };
                // Create chunk
                let chunk = processor.process_file_as_chunk(&file_path, container_type)?;

                // Reset flow control per processing
                if processor.state.flow_control != FlowControl::None {
                    processor.reset_flow_control();
                }
                if raw_include {
                    processor.state.flow_control = FlowControl::None;
                }
                processor.set_sandbox(false);
                processor.state.input_stack.remove(&canonic); // Collect stack
                Ok(chunk)
            } else {
                let formatted = format!(
                    "File path : \"{}\" doesn't exist or not a file",
                    file_path.display()
                );
                Err(RadError::InvalidArgument(formatted))
            }
        } else {
            Err(RadError::InvalidArgument(
                "Include requires an argument".to_owned(),
            ))
        }
    }

    /// Paste given file's content but always read
    ///
    /// Every macros within the file is also expanded
    ///
    /// Include read file's content into a single string and print out.
    /// This enables ergonomic process of macro execution. If you want file
    /// inclusion to happen as read, use incread instead.
    ///
    /// # Usage
    ///
    /// $incread(path)
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn incread(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("incread", AuthType::FIN, processor)? {
            return Ok(None);
        }
        if !args.is_empty() {
            Ok(processor.execute_macro(level, "incread", "include", args)?)
        } else {
            Err(RadError::InvalidArgument(
                "Include requires an argument".to_owned(),
            ))
        }
    }

    /// Include but for temporary file
    ///
    /// This reads file's content into memory. Use read macro if streamed write is needed.
    ///
    /// # Usage
    ///
    /// $tempin()
    #[cfg(not(feature = "wasm"))]
    pub(crate) fn temp_include(
        _: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let file = processor.get_temp_path().display();
        let chunk = Self::include(&file.to_string(), level, processor)?;
        Ok(chunk)
    }

    #[allow(unused_variables)]
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub(crate) fn test_logics(
        args: &str,
        level: usize,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().no_strip().args_with_len(args, 3) {
            //processor.log_message(&args[0]);
            //processor.log_message(&args[1]);
            //processor.log_message(&args[2]);
            Ok(None)
        } else {
            Err(RadError::InvalidArgument(
                "Insufficient argumetns for test".to_owned(),
            ))
        }
    }
    // Keyword macros end
    // ----------
}
