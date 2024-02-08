use crate::auth::AuthType;
use crate::common::{ContainerType, FileTarget, FlowControl, MacroAttribute, ProcessInput};
use crate::common::{ErrorBehaviour, MacroType, RadResult, RelayTarget, STREAM_CONTAINER};
use crate::consts::MACRO_SPECIAL_ANON;
use crate::deterred_map::DeterredMacroMap;
use crate::formatter::Formatter;
use crate::parser::SplitVariant;
use crate::utils::{Utils, NUM_MATCH};
use crate::NewArgParser as ArgParser;
use crate::WarningType;
use crate::{Processor, RadError};
use dcsv::VCont;
use evalexpr::eval;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
        _: &MacroAttribute,
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
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("append", &args, attr, 2, Some(&mut ap))?;

        let name = processor.parse_and_strip(&mut ap, attr, level, "append", args[0].trim())?;
        let target = processor.parse_and_strip(&mut ap, attr, level, "append", args[1].as_ref())?;

        if let Some(name) = processor.contains_local_macro(level, &name) {
            processor.append_local_macro(&name, &target);
        } else if processor.contains_macro(&name, MacroType::Runtime) {
            processor.append_macro(&name, &target);
        } else {
            return Err(RadError::InvalidArgument(format!(
                "Macro \"{}\" doesn't exist",
                name
            )));
        }

        Ok(None)
    }

    /// Apply map on array
    ///
    /// # Usage
    ///
    /// $map(macro_name,array)
    pub(crate) fn map_array(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("map", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let expanded = p.parse_and_strip(&mut ap, attr, level, "map", args[0].trim())?;
        let (name, arguments) = Utils::get_name_n_arguments(&expanded, true)?;
        let src = p.parse_and_strip(&mut ap, attr, level, "map", &args[1])?;
        let array = src.split(',');

        let mut acc = String::new();
        for item in array {
            acc.push_str(
                &p.execute_macro(level, "map", name, &(arguments.clone() + item))?
                    .unwrap_or_default(),
            );
        }
        Ok(Some(acc))
    }

    /// Apply map on lines
    ///
    /// # Usage
    ///
    /// $mapl(macro_name,lines)
    pub(crate) fn map_lines(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("mapl", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let expanded = p.parse_and_strip(&mut ap, attr, level, "mapl", args[0].trim())?;
        let (name, arguments) = Utils::get_name_n_arguments(&expanded, true)?;
        let src = p.parse_and_strip(&mut ap, attr, level, "mapl", &args[1])?;
        let lines = src.lines();

        let mut acc = String::new();
        for item in lines {
            acc.push_str(
                &p.execute_macro(level, "mapl", name, &(arguments.clone() + item))?
                    .unwrap_or_default(),
            );
        }
        Ok(Some(acc))
    }

    /// Apply map on numbers
    ///
    /// # Usage
    ///
    /// $mapn()
    pub(crate) fn map_number(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("mapn", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let mut operation = String::new();
        let op_src = p.parse_and_strip(&mut ap, attr, level, "mapn", args[0].trim())?;
        let src = &p.parse_and_strip(&mut ap, attr, level, "mapn", args[1].trim())?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&op_src, true)?;

        let map_type = if p.contains_macro(macro_name, MacroType::Any) {
            "macro"
        } else if operation.contains('n') {
            "formula"
        } else {
            operation = op_src.to_string();
            operation.insert(0, 'n');
            "formula"
        };

        let mut new = String::with_capacity(src.len());
        let mut last_match = 0;
        for caps in NUM_MATCH.captures_iter(src) {
            let m = caps.get(0).unwrap();
            new.push_str(&src[last_match..m.start()]);
            let evaluated = match map_type {
                "macro" => p
                    .execute_macro(
                        level,
                        "mapn",
                        macro_name,
                        &(macro_arguments.clone() + m.as_str()),
                    )?
                    .unwrap_or_default(),
                "formula" => eval(&operation.replace('n', m.as_str()))?.to_string(),
                _ => unreachable!(),
            };
            new.push_str(&evaluated);
            last_match = m.end();
        }
        new.push_str(&src[last_match..]);
        Ok(Some(new))
    }

    /// Apply map on file lines
    ///
    /// # Usage
    ///
    /// $mapf(macro_name,file_name)
    pub(crate) fn map_file(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("mapf", AuthType::FIN, p)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("mapf", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let macro_src = p.parse_and_strip(&mut ap, attr, level, "mapf", args[0].trim())?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let file = BufReader::new(std::fs::File::open(
            p.parse_and_strip(&mut ap, attr, level, "mapf", args[1].trim())?
                .as_ref(),
        )?)
        .lines();

        let mut acc = String::new();
        for line in file {
            let line = line?;
            acc.push_str(
                &p.execute_macro(
                    level,
                    "mapf",
                    macro_name,
                    &(macro_arguments.clone() + &line),
                )?
                .unwrap_or_default(),
            );
        }
        Ok(Some(acc))
    }

    /// Apply maps on captured expressions
    ///
    /// # Usage
    ///
    /// $grepmap(type,expr,macro,text)
    pub(crate) fn grep_map(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("grepmap", &args, attr, 4, Some(&mut ap))?;

        ap.set_strip(true);
        let grep_type = p.parse_and_strip(&mut ap, attr, level, "grepmap", args[0].trim())?;
        let match_expr = p.parse_and_strip(&mut ap, attr, level, "grepmap", &args[1])?;
        let macro_src = p.parse_and_strip(&mut ap, attr, level, "grepmap", args[2].trim())?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let source = p.parse_and_strip(&mut ap, attr, level, "grepmap", &args[3])?;

        let bufread = match grep_type.to_lowercase().as_str() {
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

        if bufread && !std::path::Path::new(source.as_ref()).exists() {
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
        let reg = p.try_get_or_insert_regex(&match_expr)?.clone();

        if !bufread {
            for cap in reg.captures_iter(&source) {
                let captured = cap.get(0).map_or("", |m| m.as_str());
                let expanded = p
                    .execute_macro(
                        level,
                        "grepmap",
                        macro_name,
                        &(macro_arguments.clone() + captured),
                    )?
                    .unwrap_or_default();
                res.push_str(&expanded);
            }
        } else {
            let lines = BufReader::new(File::open(std::path::Path::new(source.as_ref()))?).lines();

            for line in lines {
                let line = line?;
                for cap in reg.captures_iter(&line) {
                    let captured = cap.get(0).map_or("", |m| m.as_str());
                    let expanded = p
                        .execute_macro(
                            level,
                            "grepmap",
                            macro_name,
                            &(macro_arguments.clone() + captured),
                        )?
                        .unwrap_or_default();
                    res.push_str(&expanded);
                }
            }
        }

        Ok(Some(res))
    }

    /// Loop around given values which is separated by given separator
    ///
    /// # Usage
    ///
    /// $forby($:(),-,a-b-c)
    pub(crate) fn forby(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("forby", &args, attr, 3, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();
        let body = &args[0];
        let sep = &processor.parse_and_strip(&mut ap, attr, level, "forby", &args[1])?;
        let loopable = &processor.parse_and_strip(&mut ap, attr, level, "forby", &args[2])?;
        for (count, value) in loopable.split_terminator(sep.as_ref()).enumerate() {
            // This overrides value
            processor.add_new_local_macro(level, "a_LN", &count.to_string());
            processor.add_new_local_macro(level, ":", value);
            let result = &processor.parse_and_strip(&mut ap, attr, level, "forby", body)?;

            sums.push_str(result);
        }

        // Clear local macro
        processor.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $foreach($:(),a,b,c)
    pub(crate) fn foreach(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("foreach", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();
        let body = &args[0];

        if args[1].contains("$:()") || args[1].contains("$a_LN()") {
            processor.log_warning(
                "Foreach's second argument is iterable array.",
                WarningType::Sanity,
            )?;
        }

        let loop_src = processor.parse_and_strip(&mut ap, attr, level, "foreach", &args[1])?;
        let loopable = loop_src.trim();
        for (count, value) in loopable.split(',').enumerate() {
            // This overrides value
            processor.add_new_local_macro(level, "a_LN", &count.to_string());
            processor.add_new_local_macro(level, ":", value);
            let result = &processor.parse_and_strip(&mut ap, attr, level, "foreach", body)?;

            sums.push_str(result);
        }

        // Clear local macro
        processor.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given joined and substitute iterators with the value
    ///
    /// # Usage
    ///
    /// $forjoin($:(),a,b,c\nd,e,f)
    pub(crate) fn forjoin(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("forjoin", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();
        let body = &args[0];

        if args[1].contains("$:()") || args[1].contains("$a_LN()") {
            processor.log_warning(
                "Forjoin's second argument is iterable array.",
                WarningType::Sanity,
            )?;
        }

        let loop_src = processor.parse_and_strip(&mut ap, attr, level, "forjoin", &args[1])?;
        let loopable = loop_src.trim().lines().collect::<Vec<_>>();

        let mut line_width = 0;
        let loopable: RadResult<Vec<Vec<&str>>> = loopable
            .into_iter()
            .map(|s| {
                let splitted = s.split(',').collect::<Vec<_>>();
                if splitted.is_empty() {
                    return Err(RadError::InvalidArgument(format!(
                        "Forjoin cannot process {} as valid array",
                        s
                    )));
                } else if line_width == 0 {
                    // Initial state
                    line_width = splitted.len();
                } else if line_width != splitted.len() {
                    return Err(RadError::InvalidArgument(format!(
                        "Line {} has inconsistent array length",
                        s
                    )));
                }
                Ok(splitted)
            })
            .collect();
        let loopable = loopable?;
        for (count, value) in loopable.into_iter().enumerate() {
            processor.add_new_local_macro(level, ":0", &value.join(","));
            // This overrides value
            processor.add_new_local_macro(level, "a_LN", &count.to_string());
            for (idx, item) in value.iter().enumerate() {
                processor.add_new_local_macro(level, &format!(":{}", idx + 1), item);
            }
            let result = &processor.parse_and_strip(&mut ap, attr, level, "forjoin", body)?;

            sums.push_str(result);
        }

        // Clear local macro
        processor.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given words and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $forsp($:(),a,b,c)
    pub(crate) fn for_space(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("forsp", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();
        let body = &args[0];

        if args[1].contains("$:()") || args[1].contains("$a_LN()") {
            processor.log_warning(
                "Foreach's second argument is iterable array.",
                WarningType::Sanity,
            )?;
        }

        let loop_src = processor.parse_and_strip(&mut ap, attr, level, "forsp", &args[1])?;
        let loopable = loop_src.trim();
        for (count, value) in loopable.split_whitespace().enumerate() {
            // This overrides value
            processor.add_new_local_macro(level, "a_LN", &count.to_string());
            processor.add_new_local_macro(level, ":", value);
            let result = &processor.parse_and_strip(&mut ap, attr, level, "forsp", body)?;

            sums.push_str(result);
        }

        // Clear local macro
        processor.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given values split by new line and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $forline($:(),Content)
    pub(crate) fn forline(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("forline", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();
        let body = &args[0];
        let loop_src = processor.parse_and_strip(&mut ap, attr, level, "forline", &args[1])?;
        let mut count = 1;
        for line in Utils::full_lines(&loop_src) {
            // This overrides value
            processor.add_new_local_macro(level, "a_LN", &count.to_string());
            processor.add_new_local_macro(level, ":", line);
            let result = processor.parse_and_strip(&mut ap, attr, level, "forline", body)?;
            sums.push_str(&result);
            count += 1;
        }
        Ok(Some(sums))
    }

    /// For loop around given min, max value and finally substitue iterators with value
    ///
    /// # Usage
    ///
    /// $forloop($:(),1,5)
    pub(crate) fn forloop(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("forloop", &args, attr, 3, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();

        if args[2].contains("$:()") || args[1].contains("$a_LN()") {
            processor.log_warning(
                "Forloop's third argument is a max number",
                WarningType::Sanity,
            )?;
        }

        let body = &args[0];
        let min_src = &processor
            .parse_and_strip(&mut ap, attr, level, "forloop", &args[1])?
            .trim()
            .to_string();
        let max_src = &processor
            .parse_and_strip(&mut ap, attr, level, "forloop", &args[2])?
            .trim()
            .to_string();

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
        let mut counter = 1;
        for value in min..=max {
            processor.add_new_local_macro(level, ":", &value.to_string());
            processor.add_new_local_macro(level, "a_LN", &counter.to_string());
            result = processor
                .parse_and_strip(&mut ap, attr, level, "forloop", body)?
                .to_string();

            sums.push_str(&result);
            result.clear();
            counter += 1;
        }

        // Clear local macro
        processor.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around table columns
    ///
    /// # Usage
    ///
    /// $forcol($:(),Content)
    pub(crate) fn forcol(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("forcol", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let mut sums = String::new();
        let body = &args[0];
        let loop_src = processor.parse_and_strip(&mut ap, attr, level, "forcol", &args[1])?;
        let table = dcsv::Reader::new()
            .trim(true)
            .has_header(false)
            .use_space_delimiter(true)
            .array_from_stream(loop_src.as_bytes())?;
        for idx in 0..table.get_column_count() {
            let mut column = table.get_column_iterator(idx)?.map(|s| s.to_string()).fold(
                String::new(),
                |mut acc, v| {
                    acc.push_str(&v.to_string());
                    acc.push(',');
                    acc
                },
            );
            column.pop();
            // This overrides value
            processor.add_new_local_macro(level, "a_LN", &idx.to_string());
            processor.add_new_local_macro(level, ":", &column);
            let result = processor.parse_and_strip(&mut ap, attr, level, "forcol", body)?;
            sums.push_str(&result);
        }
        Ok(Some(sums))
    }

    /// Log macro information
    ///
    /// # Usage
    ///
    /// $logm(mac)
    pub(crate) fn log_macro_info(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let macro_name = &p
            .parse_and_strip(&mut ap, attr, level, "logm", args)?
            .trim()
            .to_string();
        let body = if let Some(name) = p.contains_local_macro(level, macro_name) {
            p.get_local_macro_body(&name)?.to_string()
        } else if let Ok(body) = p.get_runtime_macro_body(macro_name) {
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
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("if", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let boolean = &processor.parse_and_strip(&mut ap, attr, level, "if", &args[0])?;

        // Given condition is true
        let cond = Utils::is_arg_true(boolean);
        if let Ok(cond) = cond {
            if cond {
                let if_expr = processor
                    .parse_and_strip(&mut ap, attr, level, "if", &args[1])?
                    .to_string();
                return Ok(Some(if_expr));
            }
        } else {
            return Err(RadError::InvalidArgument(format!(
                "If requires either true/false or zero/nonzero integer but given \"{}\"",
                boolean
            )));
        }

        Ok(None)
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $ifelse(evaluation, \*ifstate*\, \*elsestate*\)
    pub(crate) fn ifelse(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("ifelse", &args, attr, 3, Some(&mut ap))?;

        ap.set_strip(true);

        let boolean = &processor.parse_and_strip(&mut ap, attr, level, "ifelse", &args[0])?;

        // Given condition is true
        let cond = Utils::is_arg_true(boolean);
        if let Ok(cond) = cond {
            if cond {
                let if_expr = processor
                    .parse_and_strip(&mut ap, attr, level, "ifelse", &args[1])?
                    .to_string();
                return Ok(Some(if_expr));
            }
        } else {
            return Err(RadError::InvalidArgument(format!(
                "Ifelse requires either true/false or zero/nonzero integer but given \"{}\"",
                boolean
            )));
        }

        // Else state
        let else_expr = processor
            .parse_and_strip(&mut ap, attr, level, "ifelse", &args[2])?
            .to_string();
        Ok(Some(else_expr))
    }

    /// If macro exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifdef(macro_name, expr)
    pub(crate) fn ifdef(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("ifdef", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);

        let name = &processor
            .parse_and_strip(&mut ap, attr, level, "ifdef", &args[0])?
            .trim()
            .to_string();

        let boolean = processor.contains_macro(name, MacroType::Any);
        // Return true or false by the definition
        if boolean {
            let if_expr = processor
                .parse_and_strip(&mut ap, attr, level, "ifdef", &args[1])?
                .to_string();
            return Ok(Some(if_expr));
        }
        Ok(None)
    }

    /// If macro exists, then execute expresion else exectue another
    ///
    /// # Usage
    ///
    /// $ifdefel(macro_name,expr,expr2)
    pub(crate) fn ifdefel(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("ifdefel", &args, attr, 3, Some(&mut ap))?;

        ap.set_strip(true);

        let name = &processor
            .parse_and_strip(&mut ap, attr, level, "ifdefel", &args[0])?
            .trim()
            .to_string();

        let boolean = processor.contains_macro(name, MacroType::Any);
        // Return true or false by the definition
        if boolean {
            let if_expr = processor
                .parse_and_strip(&mut ap, attr, level, "ifdefel", &args[1])?
                .to_string();
            Ok(Some(if_expr))
        } else {
            let else_expr = processor
                .parse_and_strip(&mut ap, attr, level, "ifdefel", &args[2])?
                .to_string();
            Ok(Some(else_expr))
        }
    }

    /// If env exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifenv(env_name, expr)
    pub(crate) fn ifenv(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenv", AuthType::ENV, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("ifenv", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);

        let name = &processor
            .parse_and_strip(&mut ap, attr, level, "ifenv", &args[0])?
            .trim()
            .to_string();

        let boolean = std::env::var(name).is_ok();

        // Return true or false by the definition
        if boolean {
            let if_expr = processor
                .parse_and_strip(&mut ap, attr, level, "ifenv", &args[1])?
                .to_string();
            return Ok(Some(if_expr));
        }
        Ok(None)
    }

    /// If env exists, then execute expresion else execute another
    ///
    /// # Usage
    ///
    /// $ifenvel(env_name,expr,expr2)
    pub(crate) fn ifenvel(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenvel", AuthType::ENV, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("ifenvel", &args, attr, 3, Some(&mut ap))?;

        ap.set_strip(true);

        let name = &processor
            .parse_and_strip(&mut ap, attr, level, "ifenvel", &args[0])?
            .trim()
            .to_string();

        let boolean = std::env::var(name).is_ok();

        // Return true or false by the definition
        if boolean {
            let if_expr = processor
                .parse_and_strip(&mut ap, attr, level, "ifenvel", &args[1])?
                .to_string();
            Ok(Some(if_expr))
        } else {
            let else_expr = processor
                .parse_and_strip(&mut ap, attr, level, "ifenvel", &args[2])?
                .to_string();
            Ok(Some(else_expr))
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
        _: &MacroAttribute,
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
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let backup = processor.state.behaviour;
        processor.state.behaviour = ErrorBehaviour::Assert;

        let mut ap = ArgParser::new().no_strip();
        let result = processor.parse_and_strip(&mut ap, attr, level, "fassert", args);
        processor.state.behaviour = backup;
        processor.logger.stop_last_tracker();
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
    pub(crate) fn consume(
        _: &str,
        level: usize,
        _: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        // Eagerly pop relay target
        p.state.relay.pop();

        let mut ap = ArgParser::new().no_strip();
        ap.set_strip(true);
        let macro_src = std::mem::take(&mut p.state.stream_state.macro_src);
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;

        let body = p.extract_runtime_macro_body(STREAM_CONTAINER)?;
        let content = &body;

        let result = if p.state.stream_state.as_lines {
            let mut acc = String::new();
            for item in content.lines() {
                acc.push_str(
                    &p.execute_macro(
                        level,
                        "consume",
                        macro_name,
                        &(macro_arguments.clone() + item),
                    )?
                    .unwrap_or_default(),
                );
            }
            Some(acc)
        } else {
            p.execute_macro(
                level,
                "consume",
                macro_name,
                &(macro_arguments.clone() + content),
            )?
        };

        p.state.stream_state.clear();
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
        _: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        processor.insert_queue(args);
        Ok(None)
    }

    /// Queue processing
    ///
    /// # Usage
    ///
    /// $queif(true,Sentence to process)
    pub(crate) fn if_queue_content(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("queif", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let boolean = &processor.parse_and_strip(&mut ap, attr, level, "queif", &args[0])?;
        let cond = Utils::is_arg_true(boolean)?;
        if cond {
            processor.insert_queue(&args[1]);
        }
        Ok(None)
    }

    pub(crate) fn escape_blanks(
        _: &str,
        level: usize,
        _: &MacroAttribute,
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
    pub(crate) fn read_to(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        // Needs both permission
        if !Utils::is_granted("readto", AuthType::FIN, processor)?
            || !Utils::is_granted("readto", AuthType::FOUT, processor)?
        {
            return Ok(None);
        }
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("readto", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);
        let file_path = PathBuf::from(
            processor
                .parse_and_strip(&mut ap, attr, level, "readto", args[0].trim())?
                .as_ref(),
        );
        let to_path = PathBuf::from(
            processor
                .parse_and_strip(&mut ap, attr, level, "readto", args[1].trim())?
                .as_ref(),
        );
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
                    attr,
                    level,
                    "readto",
                    args[2].trim(),
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
    }

    /// Read in
    ///
    /// # Usage
    ///
    /// $readin(file_a)
    pub(crate) fn read_in(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("readin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("readin", &args, attr, 1, Some(&mut ap))?;

        ap.set_strip(true);
        let file_path = PathBuf::from(
            processor
                .parse_and_strip(&mut ap, attr, level, "readin", args[0].trim())?
                .as_ref(),
        );
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
                    attr,
                    level,
                    "readin",
                    args[1].trim(),
                )?)?;

                // You don't have to backup pause state because include wouldn't be triggered
                // at the first place, if paused was true
                if raw_include {
                    processor.state.flow_control = FlowControl::Escape;
                }
            }

            if let Some(relay) = processor.state.relay.last() {
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
    }

    /// Execute macro
    ///
    /// # Usage
    ///
    /// $exec(macro_name,macro_args)
    pub(crate) fn execute_macro(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("exec", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);

        let macro_name = &processor
            .parse_and_strip(&mut ap, attr, level, "exec", &args[0])?
            .trim()
            .to_string();
        let args = processor.parse_and_strip(&mut ap, attr, level, "exec", &args[1])?;
        let result = processor
            .execute_macro(level, "exec", macro_name, &args)?
            .unwrap_or_default();
        Ok(Some(result))
    }

    /// Create multiple macro executions from given csv value
    ///
    /// # Usage
    ///
    /// $spread(macro_name,\*1,2,3
    /// 4,5,6*\)
    ///
    /// $spread+(macro_name,
    /// 1,2,3
    /// 4,5,6
    /// )
    pub(crate) fn spread_data(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        let mut ap = ArgParser::new();
        let args = Utils::get_split_arguments_or_error("spread", &args, attr, 2, Some(&mut ap))?;

        ap.set_strip(true);

        let expanded_src = &processor.parse_and_strip(&mut ap, attr, level, "spread", &args[0])?;
        let expanded_data = &processor.parse_and_strip(&mut ap, attr, level, "spread", &args[1])?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(expanded_src, true)?;
        let macro_data = expanded_data.trim();

        let result = Formatter::csv_to_macros(
            macro_name,
            macro_arguments,
            macro_data,
            &processor.state.newline,
        )?;

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
        let result = processor.parse_and_strip(&mut ap, attr, level, "spread", &result)?;

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

        Ok(Some(result.to_string()))
    }

    /// stream
    ///
    /// # Usage
    ///
    /// $stream(macro_name)
    /// $consume()
    pub(crate) fn stream(
        args_src: &str,
        _: usize,
        _: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if p.state.stream_state.on_stream {
            return Err(RadError::InvalidArgument(
                "Stream cannot be nested".to_string(),
            ));
        }
        p.state.stream_state.on_stream = true;

        let name = args_src.trim();

        if name.is_empty() {
            return Err(RadError::InvalidArgument(
                "stream requires an argument".to_owned(),
            ));
        }

        p.log_warning("Streaming text content to a macro", WarningType::Security)?;

        p.state.stream_state.macro_src = name.to_string();

        p.add_container_macro(STREAM_CONTAINER)?;
        let rtype = RelayTarget::Macro(STREAM_CONTAINER.to_string());

        p.state.relay.push(rtype);
        Ok(None)
    }

    /// stream by lines
    ///
    /// # Usage
    ///
    /// $streaml(macro_name)
    /// $consume()
    pub(crate) fn stream_by_lines(
        args_src: &str,
        _: usize,
        _: &MacroAttribute,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if p.state.stream_state.on_stream {
            return Err(RadError::InvalidArgument(
                "Stream series cannot be nested".to_string(),
            ));
        }
        p.state.stream_state.on_stream = true;
        p.state.stream_state.as_lines = true;

        let name = args_src.trim();

        if name.is_empty() {
            return Err(RadError::InvalidArgument(
                "streaml requires an argument ( macro name )".to_owned(),
            ));
        }

        p.log_warning("Streaming text content to a macro", WarningType::Security)?;

        p.state.stream_state.macro_src = name.to_string();

        p.add_container_macro(STREAM_CONTAINER)?;
        let rtype = RelayTarget::Macro(STREAM_CONTAINER.to_string());

        p.state.relay.push(rtype);
        Ok(None)
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
    pub(crate) fn include(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("include", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        let args = ap.args_to_vec(args, attr, b',', SplitVariant::Always);
        ap.set_strip(true);
        if !args.is_empty() {
            let mut file_path = PathBuf::from(
                &processor
                    .parse_and_strip(&mut ap, attr, level, "include", &args[0])?
                    .trim(),
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
                        &processor.parse_and_strip(&mut ap, attr, level, "include", &args[1])?,
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
    pub(crate) fn incread(
        args: &str,
        level: usize,
        _: &MacroAttribute,
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
    pub(crate) fn temp_include(
        _: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempin", AuthType::FIN, processor)? {
            return Ok(None);
        }
        let file = processor.get_temp_path().display();
        let chunk = Self::include(&file.to_string(), level, attr, processor)?;
        Ok(chunk)
    }

    #[allow(unused_variables)]
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub(crate) fn test_logics(
        args: &str,
        level: usize,
        attr: &MacroAttribute,
        processor: &mut Processor,
    ) -> RadResult<Option<String>> {
        if let Some(args) = ArgParser::new().no_strip().args_with_len(args, attr, 3) {
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
