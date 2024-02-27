use crate::argument::MacroInput;
use crate::auth::AuthType;
use crate::common::{ContainerType, FileTarget, FlowControl, MacroFragment, ProcessInput};
use crate::common::{ErrorBehaviour, MacroType, RadResult, RelayTarget, STREAM_CONTAINER};
use crate::consts::MACRO_SPECIAL_ANON;
use crate::deterred_map::DeterredMacroMap;
use crate::formatter::Formatter;
use crate::parser::ArgParser;
use crate::utils::{RadStr, Utils, NUM_MATCH};
use crate::WarningType;
use crate::{Processor, RadError};
use dcsv::VCont;
use evalexpr::eval;
use itertools::Itertools;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let args = p.parse_chunk(level, "anon", input.args)?;
        p.add_anon_macro(&args)?;
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
    /// $append(macro_name,Content)
    pub(crate) fn append(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("append")
            .cursors_with_len(input)?;

        let name = cursors.get_text(p, 0)?;
        let target = cursors.get_text(p, 1)?;

        if let Some(name) = p.contains_local_macro(level, &name) {
            p.append_local_macro(&name, &target);
        } else if p.contains_macro(&name, MacroType::Runtime) {
            p.append_macro(&name, &target);
        } else {
            return Err(RadError::NoSuchMacroName(
                name.clone(),
                p.get_similar_macro(&name, true), // Only runtime
            ));
        }

        Ok(None)
    }

    /// Apply map on expressions
    ///
    /// # Usage
    ///
    /// $map(expr,macro,text)
    pub(crate) fn map(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("map")
            .cursors_with_len(input)?;

        let match_expr = cursors.get_text(p, 0)?;
        let macro_src = cursors.get_ctext(p, 1)?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let source = cursors.get_text(p, 2)?;

        if match_expr.is_empty() {
            return Err(RadError::InvalidArgument(
                "Regex expression cannot be an empty string".to_string(),
            ));
        }

        let mut res = String::new();

        let reg = p.try_get_or_insert_regex(&match_expr)?.clone();

        append_captured_expressions(
            &reg,
            &source,
            &mut res,
            p,
            level,
            "map",
            macro_name,
            &macro_arguments,
        )?;

        Ok(Some(res))
    }

    /// Apply map on array
    ///
    /// # Usage
    ///
    /// $mapa(macro_name,array)
    pub(crate) fn map_array(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("mapa")
            .cursors_with_len(input)?;

        let expanded = cursors.get_text(p, 0)?;
        let (name, arguments) = Utils::get_name_n_arguments(&expanded, true)?;
        let src = cursors.get_text(p, 1)?;
        let array = src.split(',');

        let mut acc = String::new();
        for item in array {
            acc.push_str(
                &p.execute_macro(level, "mapa", name, &(arguments.clone() + item))?
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("mapl")
            .cursors_with_len(input)?;

        let expanded = cursors.get_text(p, 0)?;
        let (name, arguments) = Utils::get_name_n_arguments(&expanded, true)?;
        let src = cursors.get_text(p, 1)?;
        let lines = src.full_lines();

        let mut acc = String::new();
        for item in lines {
            acc.push_str(
                &p.execute_macro(level, "mapl", name, &(arguments.clone() + item))?
                    .unwrap_or_default(),
            );
        }
        Ok(Some(acc))
    }

    /// Apply maps on captured expressions from lines
    ///
    /// # Usage
    ///
    /// $maple(type,expr,macro,text)
    pub(crate) fn map_lines_expr(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("maple")
            .cursors_with_len(input)?;

        let match_expr = cursors.get_text(p, 0)?;
        let macro_src = cursors.get_text(p, 1)?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let source = cursors.get_text(p, 2)?;

        if match_expr.is_empty() {
            return Err(RadError::InvalidArgument(
                "Regex expression cannot be an empty string".to_string(),
            ));
        }

        let mut res = String::new();

        // If this regex is not cloned, "capture" should collect captured string into a allocated
        // vector. Which is generaly worse for performance.
        let reg = p.try_get_or_insert_regex(&match_expr)?.clone();

        let preserve_non_matched_lines = p.env.map_preserve;

        let mut captured;
        for line in source.full_lines() {
            captured = append_captured_expressions(
                &reg,
                line,
                &mut res,
                p,
                level,
                "maple",
                macro_name,
                &macro_arguments,
            )?;

            if preserve_non_matched_lines && !captured {
                res.push_str(line);
            }
        }

        Ok(Some(res))
    }

    /// Map on expression chunk
    ///
    /// # Usage
    ///
    /// $mape(expr,1
    /// 2)
    pub(crate) fn map_expression(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("mape")
            .cursors_with_len(input)?;

        let start_expr = cursors.get_text(p, 0)?;
        let end_expr = cursors.get_text(p, 1)?;
        let macro_src = cursors.get_text(p, 2)?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let source = cursors.get_text(p, 3)?;

        if start_expr.is_empty() || end_expr.is_empty() {
            return Err(RadError::InvalidArgument(
                "Regex expression cannot be an empty string".to_string(),
            ));
        }

        let mut chunk_index = 0usize;
        let mut folded = String::with_capacity(source.len());
        let preserve = p.env.map_preserve;
        let mut on_grep = false;

        let regs = p.try_get_or_insert_multiple_regex(&[&start_expr, &end_expr])?;
        let reg_start = regs[0].clone();
        let reg_end = regs[1].clone();

        let mut iter = source.full_lines_with_index().peekable();
        while let Some((idx, line)) = iter.next() {
            // Start a new container
            if reg_start.find(line).is_some() && reg_end.find(line).is_none() {
                let previous = &source[chunk_index..idx.saturating_sub(1)];
                on_grep = true;
                if !previous.is_empty() {
                    folded.push_str(
                        &p.execute_macro(
                            level,
                            "mape",
                            macro_name,
                            &(macro_arguments.clone() + previous),
                        )?
                        .unwrap_or_default(),
                    );
                }
                chunk_index = idx;
            } else if reg_start.find(line).is_none() && reg_end.find(line).is_some() {
                let last_index = iter.peek().unwrap_or(&(source.len() - 1, "")).0;
                let current = &source[chunk_index..last_index];
                if !current.is_empty() {
                    folded.push_str(
                        &p.execute_macro(
                            level,
                            "mape",
                            macro_name,
                            &(macro_arguments.clone() + current),
                        )?
                        .unwrap_or_default(),
                    );
                }
                chunk_index = last_index + 1;
                on_grep = false;
            } else if preserve && !on_grep {
                folded.push_str(line);
            }
        }

        Ok(Some(folded))
    }

    /// Apply map on numbers
    ///
    /// # Usage
    ///
    /// $mapn()
    pub(crate) fn map_number(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("mapn")
            .cursors_with_len(input)?;

        let mut operation = String::new();
        let op_src = cursors.get_ctext(p, 0)?;
        let src = cursors.get_text(p, 1)?;
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
        for caps in NUM_MATCH.captures_iter(&src) {
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("mapf", AuthType::FIN, p)? {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("mapf")
            .cursors_with_len(input)?;

        let macro_src = cursors.get_ctext(p, 0)?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let file = Utils::full_lines(BufReader::new(std::fs::File::open(
            cursors.get_path(p, 1)?,
        )?));

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

    /// Apply maps on captured expressions frome file
    ///
    /// # Usage
    ///
    /// $mapfe(type,expr,macro,text)
    pub(crate) fn map_file_expr(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("mapfe", AuthType::FIN, p)? {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("mapfe")
            .cursors_with_len(input)?;

        let match_expr = cursors.get_text(p, 0)?;
        let macro_src = cursors.get_ctext(p, 1)?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
        let source_file = cursors.get_path(p, 2)?;

        if !source_file.exists() {
            return Err(RadError::InvalidExecution(format!(
                "Cannot find a file \"{}\" ",
                source_file.display()
            )));
        }

        let mut res = String::new();

        // If this regex is not cloned, "capture" should collect captured string into a allocated
        // vector. Which is generaly worse for performance.
        let reg = p.try_get_or_insert_regex(&match_expr)?.clone();

        let preserve_non_matched_lines = p.env.map_preserve;

        let lines = Utils::full_lines(BufReader::new(File::open(source_file)?));

        for line in lines {
            let line = line?;
            let captured = append_captured_expressions(
                &reg,
                &line,
                &mut res,
                p,
                level,
                "mapfe",
                macro_name,
                &macro_arguments,
            )?;
            if preserve_non_matched_lines && !captured {
                res.push_str(&line);
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("forby")
            .cursors_with_len(input)?;

        let mut sums = String::new();
        let sep = cursors.get_text(p, 1)?;
        let loopable = cursors.get_text(p, 2)?;
        for (count, value) in loopable.split_terminator(&sep).enumerate() {
            // This overrides value
            p.add_new_local_macro(level, "a_LN", &count.to_string());
            p.add_new_local_macro(level, ":", value);
            let result = cursors.get_text(p, 0)?;

            sums.push_str(&result);
        }

        // Clear local macro
        p.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $foreach($:(),a,b,c)
    pub(crate) fn foreach(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("foreach")
            .cursors_with_len(input)?;

        let mut sums = String::new();

        // TODO TT
        // if args[1].contains("$:()") || args[1].contains("$a_LN()") {
        //     p.log_warning(
        //         "Foreach's second argument is iterable array.",
        //         WarningType::Sanity,
        //     )?;
        // }

        let loop_src = cursors.get_text(p, 1)?;
        let loopable = loop_src.trim();
        for (count, value) in loopable.split(',').enumerate() {
            // This overrides value
            p.add_new_local_macro(level, "a_LN", &count.to_string());
            p.add_new_local_macro(level, ":", value);
            let result = cursors.get_text(p, 0)?;

            sums.push_str(&result);
        }

        // Clear local macro
        p.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given joined and substitute iterators with the value
    ///
    /// # Usage
    ///
    /// $forjoin($:(),a,b,c\nd,e,f)
    pub(crate) fn forjoin(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("forjoin")
            .cursors_with_len(input)?;

        let mut sums = String::new();

        // TODO TT
        // if args[1].contains("$:()") || args[1].contains("$a_LN()") {
        //     p.log_warning(
        //         "Forjoin's second argument is iterable array.",
        //         WarningType::Sanity,
        //     )?;
        // }

        let loop_src = cursors.get_text(p, 1)?;
        let loopable = loop_src.trim().lines().collect::<Vec<_>>();

        let mut line_width = 0;
        let loopable: RadResult<Vec<Vec<&str>>> = loopable
            .into_iter()
            .map(|s| {
                let splitted = s.split(',').collect::<Vec<_>>();
                if splitted.is_empty() {
                    return Err(RadError::InvalidArgument(format!(
                        "Forjoin cannot process {} as a valid array",
                        s
                    )));
                }
                if line_width == 0 {
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
            p.add_new_local_macro(level, ":0", &value.join(","));
            // This overrides value
            p.add_new_local_macro(level, "a_LN", &count.to_string());
            for (idx, item) in value.iter().enumerate() {
                p.add_new_local_macro(level, &format!(":{}", idx + 1), item);
            }
            let result = cursors.get_text(p, 0)?;

            sums.push_str(&result);
        }

        // Clear local macro
        p.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given words and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $forsp($:(),a,b,c)
    pub(crate) fn for_space(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("forsp")
            .cursors_with_len(input)?;

        let mut sums = String::new();

        // TODO TT
        // if args[1].contains("$:()") || args[1].contains("$a_LN()") {
        //     p.log_warning(
        //         "Foreach's second argument is iterable array.",
        //         WarningType::Sanity,
        //     )?;
        // }

        let loop_src = cursors.get_text(p, 1)?;
        let loopable = loop_src.trim();
        for (count, value) in loopable.split_whitespace().enumerate() {
            // This overrides value
            p.add_new_local_macro(level, "a_LN", &count.to_string());
            p.add_new_local_macro(level, ":", value);
            let result = cursors.get_text(p, 0)?;

            sums.push_str(&result);
        }

        // Clear local macro
        p.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around given values split by new line and substitute iterators  with the value
    ///
    /// # Usage
    ///
    /// $forline($:(),Content)
    pub(crate) fn forline(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("forline")
            .cursors_with_len(input)?;

        let mut sums = String::new();
        let loop_src = cursors.get_text(p, 1)?;
        let mut count = 1;
        for line in loop_src.full_lines() {
            // This overrides value
            p.add_new_local_macro(level, "a_LN", &count.to_string());
            p.add_new_local_macro(level, ":", line);
            let result = cursors.get_text(p, 0)?;
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("forloop")
            .cursors_with_len(input)?;

        let mut sums = String::new();

        // TODO TT
        // if args[2].contains("$:()") || args[1].contains("$a_LN()") {
        //     p.log_warning(
        //         "Forloop's third argument is a max number",
        //         WarningType::Sanity,
        //     )?;
        // }

        let min = cursors.get_uint(p, 1)?;
        let max = cursors.get_uint(p, 2)?;

        let mut counter = 1;
        for value in min..=max {
            p.add_new_local_macro(level, ":", &value.to_string());
            p.add_new_local_macro(level, "a_LN", &counter.to_string());
            let result = cursors.get_text(p, 0)?;

            sums.push_str(&result);
            // TODO TT
            // Previoulys result was mutable container outside
            // result.clear();
            counter += 1;
        }

        // Clear local macro
        p.remove_local_macro(level, ":");

        Ok(Some(sums))
    }

    /// Loop around table columns
    ///
    /// # Usage
    ///
    /// $forcol($:(),Content)
    pub(crate) fn forcol(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("forcol")
            .cursors_with_len(input)?;

        let mut sums = String::new();
        let loop_src = cursors.get_text(p, 1)?;
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
            p.add_new_local_macro(level, "a_LN", &idx.to_string());
            p.add_new_local_macro(level, ":", &column);
            let result = cursors.get_text(p, 0)?;
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("logm")
            .cursors_with_len(input)?;

        let macro_name = cursors.get_ctext(p, 0)?;
        let body = if let Some(name) = p.contains_local_macro(level, &macro_name) {
            p.get_local_macro_body(&name)?.to_string()
        } else if let Ok(body) = p.get_runtime_macro_body(&macro_name) {
            body.to_string()
        } else {
            return Err(RadError::NoSuchMacroName(
                macro_name.to_owned(),
                p.get_similar_macro(&macro_name, true), // Only runtime
            ));
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("if")
            .cursors_with_len(input)?;

        let cond = cursors.get_bool(p, 0)?;

        // Given condition is true
        if cond {
            let if_expr = cursors.get_text(p, 1)?;
            return Ok(Some(if_expr));
        }

        Ok(None)
    }

    /// Print content according to given condition
    ///
    /// # Usage
    ///
    /// $ifelse(evaluation, \*ifstate*\, \*elsestate*\)
    pub(crate) fn ifelse(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("ifelse")
            .cursors_with_len(input)?;

        let cond = cursors.get_bool(p, 0)?;

        // Given condition is true
        if cond {
            let if_expr = cursors.get_text(p, 1)?;
            return Ok(Some(if_expr));
        }

        // Else state
        let else_expr = cursors.get_text(p, 2)?;
        Ok(Some(else_expr))
    }

    /// If macro exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifdef(macro_name, expr)
    pub(crate) fn ifdef(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("ifdef")
            .cursors_with_len(input)?;

        let name = cursors.get_ctext(p, 0)?;

        let boolean = p.contains_macro(&name, MacroType::Any);
        // Return true or false by the definition
        if boolean {
            let if_expr = cursors.get_text(p, 1)?;
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("ifdefel")
            .cursors_with_len(input)?;

        let name = cursors.get_ctext(p, 0)?;

        let boolean = p.contains_macro(&name, MacroType::Any);
        // Return true or false by the definition
        if boolean {
            let if_expr = cursors.get_text(p, 1)?;
            Ok(Some(if_expr))
        } else {
            let else_expr = cursors.get_text(p, 2)?;
            Ok(Some(else_expr))
        }
    }

    /// If env exists, then execute expresion
    ///
    /// # Usage
    ///
    /// $ifenv(env_name, expr)
    pub(crate) fn ifenv(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenv", AuthType::ENV, p)? {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("ifenv")
            .cursors_with_len(input)?;

        let name = cursors.get_ctext(p, 0)?;

        let boolean = std::env::var(name).is_ok();

        // Return true or false by the definition
        if boolean {
            let if_expr = cursors.get_text(p, 1)?;
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("ifenvel", AuthType::ENV, p)? {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("ifenvel")
            .cursors_with_len(input)?;

        let name = cursors.get_ctext(p, 0)?;

        let boolean = std::env::var(name).is_ok();

        // Return true or false by the definition
        if boolean {
            let if_expr = cursors.get_text(p, 1)?;
            Ok(Some(if_expr))
        } else {
            let else_expr = cursors.get_text(p, 2)?;
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if input.args.trim().is_empty() {
            p.log_warning("Expanding empty value", WarningType::Sanity)?;
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("expand")
            .cursors_with_len(input)?;
        let result = cursors.get_text(p, 0)?;

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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let backup = p.state.behaviour;
        p.state.behaviour = ErrorBehaviour::Assert;

        let cursors = ArgParser::new()
            .level(level)
            .macro_name("if")
            .cursors_with_len(input)?;

        let prev_level = p.logger.get_tracker_level();
        let result = cursors.get_text(p, 0);
        p.state.behaviour = backup;

        // If tracker level is same, then it means plain text was supplied.
        if p.logger.get_tracker_level() != prev_level {
            p.logger.stop_last_tracker();
        }
        if result.is_err() {
            p.track_assertion(true)?;
            Ok(None)
        } else {
            p.track_assertion(false)?;
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
        _: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        // Eagerly pop relay target
        p.state.relay.pop();

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
        input: MacroInput,
        _: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if input.args.trim().is_empty() {
            p.log_warning("Queuing empty value", WarningType::Sanity)?;
        }
        p.insert_queue(input.args);
        Ok(None)
    }

    /// Queue processing
    ///
    /// # Usage
    ///
    /// $queif(true,Sentence to process)
    pub(crate) fn if_queue_content(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("queif")
            .cursors_with_len(input)?;

        let cond = cursors.get_bool(p, 0)?;
        if cond {
            let que = cursors.get_text(p, 1)?.clone();
            p.insert_queue(&que);
        }
        Ok(None)
    }

    pub(crate) fn escape_blanks(
        _: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if level != 1 {
            return Err(RadError::UnallowedMacroExecution(
                "\"EB\" is only available on first level".to_string(),
            ));
        }
        p.state.lexor_escape_blanks = true;
        Ok(None)
    }

    /// Read to
    ///
    /// # Usage
    ///
    /// $readto(file_a,file_b)
    pub(crate) fn read_to(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        // Needs both permission
        if !Utils::is_granted("readto", AuthType::FIN, p)?
            || !Utils::is_granted("readto", AuthType::FOUT, p)?
        {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("readto")
            .cursors_with_len(input)?;

        let file_path = cursors.get_path(p, 0)?;
        let to_path = cursors.get_path(p, 1)?;
        if file_path == to_path {
            return Err(RadError::InvalidArgument(format!(
                "readto cannot read from and into a same file \"{}\"",
                file_path.display()
            )));
        }
        if file_path.is_file() {
            let canonic = file_path.canonicalize()?;
            Utils::check_file_sanity(p, &canonic)?;

            // Check path sanity if to_path exists
            if to_path.exists() {
                Utils::check_file_sanity(p, &to_path.canonicalize()?)?;
            }
            // Set sandbox after error checking or it will act starngely
            p.set_sandbox(true);

            let mut reset_flow = false;
            // Optionally enable raw mode
            if let Ok(true) = cursors.get_bool(p, 2) {
                // You don't have to backup pause state because include wouldn't be triggered
                // at the first place, if paused was true
                p.state.flow_control = FlowControl::Escape;
                reset_flow = true;
            }

            let file_target = FileTarget::from_path(&to_path)?;
            p.state.relay.push(RelayTarget::File(file_target));

            // Create chunk
            let chunk = p.process_file_as_chunk(&file_path, ContainerType::Expand)?;

            // Reset flow control per processing
            if p.state.flow_control != FlowControl::None || reset_flow {
                p.reset_flow_control();
            }

            p.set_sandbox(false);
            p.state.input_stack.remove(&canonic); // Collect stack
            p.state.relay.pop(); // Pop relay
            Ok(chunk)
        } else {
            Err(RadError::InvalidExecution(format!(
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("readin", AuthType::FIN, p)? {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("readin")
            .cursors_with_len(input)?;

        let file_path = cursors.get_path(p, 0)?;
        if file_path.is_file() {
            let canonic = file_path.canonicalize()?;
            Utils::check_file_sanity(p, &canonic)?;
            // Set sandbox after error checking or it will act starngely
            p.set_sandbox(true);

            let mut reset_flow = false;
            // Optionally enable raw mode
            if let Ok(true) = cursors.get_bool(p, 1) {
                // You don't have to backup pause state because include wouldn't be triggered
                // at the first place, if paused was true
                p.state.flow_control = FlowControl::Escape;
                reset_flow = true;
            }

            if let Some(relay) = p.state.relay.last() {
                p.log_warning(
                    &format!("Read file's content will be relayed to \"{:?}\"", relay),
                    WarningType::Sanity,
                )?;
            }

            // Create chunk
            let chunk = p.process_file(&file_path)?;

            // Reset flow control per processing
            if p.state.flow_control != FlowControl::None || reset_flow {
                p.reset_flow_control();
            }

            p.set_sandbox(false);
            p.state.input_stack.remove(&canonic); // Collect stack
            Ok(chunk)
        } else {
            Err(RadError::InvalidExecution(format!(
                "readin cannot read non-file \"{}\"",
                file_path.display()
            )))
        }
    }

    /// Execute macro
    ///
    /// # Usage
    ///
    /// $exec(macro_name,attrs,macro_args)
    pub(crate) fn execute_macro(
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("exec")
            .cursors_with_len(input)?;

        let macro_name = cursors.get_ctext(p, 0)?;
        let attrs = cursors.get_ctext(p, 1)?;
        let args = cursors.get_text(p, 2)?;
        let mut frag = MacroFragment::new();
        frag.args = args.to_string();
        frag.name = macro_name.to_string();
        frag.attribute.set_from_string(&attrs);
        let result = p
            .execute_macro_with_frag(level, "exec", &mut frag)?
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("spread")
            .cursors_with_len(input)?;

        let expanded_src = cursors.get_text(p, 0)?;
        let expanded_data = cursors.get_text(p, 1)?;
        let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&expanded_src, true)?;
        let macro_data = expanded_data.trim();

        let result =
            Formatter::csv_to_macros(macro_name, macro_arguments, macro_data, &p.state.newline)?;

        // Disable debugging for nested macro expansion
        #[cfg(feature = "debug")]
        let original = p.is_debug();

        // Now this might look strange,
        // "Why not just enclose two lines with curly brackets?"
        // The answer is such appraoch somehow doesn't work.
        // Compiler cannot deduce the variable original and will yield error on
        // p.debug(original)
        #[cfg(feature = "debug")]
        p.set_debug(false);

        // Parse macros
        let result = p.parse_chunk(level, "spread", &result)?;

        // Set custom prompt log to indicate user thatn from macro doesn't support
        // debugging inside macro expansion
        #[cfg(feature = "debug")]
        {
            use crate::debugger::DebugSwitch;
            p.set_debug(original);
            match p.get_debug_switch() {
                DebugSwitch::StepMacro | DebugSwitch::NextMacro => p.set_prompt("\"Spread macro\""),
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
        input: MacroInput,
        _: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if p.state.stream_state.on_stream {
            return Err(RadError::UnallowedMacroExecution(
                "Stream cannot be nested".to_string(),
            ));
        }
        p.state.stream_state.on_stream = true;

        let ap = ArgParser::new().no_strip();
        let cursors = ap.cursors_with_len(input)?;
        let name = cursors.get_ctext(p, 0)?;

        if name.is_empty() {
            return Err(RadError::InvalidArgument(
                "stream requires an argument".to_owned(),
            ));
        }

        p.log_warning("Streaming text content to a macro", WarningType::Security)?;

        p.state.stream_state.macro_src = name;

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
        input: MacroInput,
        _: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if p.state.stream_state.on_stream {
            return Err(RadError::UnallowedMacroExecution(
                "Stream series cannot be nested".to_string(),
            ));
        }
        p.state.stream_state.on_stream = true;
        p.state.stream_state.as_lines = true;

        let ap = ArgParser::new().no_strip();
        let cursors = ap.cursors_with_len(input)?;
        let name = cursors.get_ctext(p, 0)?;

        if name.is_empty() {
            return Err(RadError::InvalidArgument(
                "streaml requires an argument ( macro name )".to_owned(),
            ));
        }

        p.log_warning("Streaming text content to a macro", WarningType::Security)?;

        p.state.stream_state.macro_src = name;

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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("include", AuthType::FIN, p)? {
            return Ok(None);
        }
        let ap = ArgParser::new().no_strip();
        let cursors = ap.cursors_with_len(input)?;
        let file_path = cursors.get_path(p, 0)?;
        let raw_include = cursors.get_bool(p, 1).unwrap_or(false);
        Self::include_inner(p, level, file_path, raw_include)
    }

    #[inline]
    fn include_inner(
        p: &mut Processor,
        level: usize,
        mut file_path: PathBuf,
        raw_include: bool,
    ) -> RadResult<Option<String>> {
        // if current input is not stdin and file path is relative
        // Create new file path that starts from current file path
        if let ProcessInput::File(path) = &p.state.current_input {
            if file_path.is_relative() {
                // It is ok get parent because any path that has a length can return parent
                file_path = path.parent().unwrap().join(file_path);
            }
        }

        if file_path.is_file() {
            let canonic = file_path.canonicalize()?;

            Utils::check_file_sanity(p, &canonic)?;
            // Set sandbox after error checking or it will act starngely
            p.set_sandbox(true);

            // Optionally enable raw mode
            if raw_include {
                p.state.flow_control = FlowControl::Escape;
            }

            let container_type = if level != 1 {
                ContainerType::Argument
            } else {
                ContainerType::Expand
            };
            // Create chunk
            let chunk = p.process_file_as_chunk(&file_path, container_type)?;

            // Reset flow control per processing
            if p.state.flow_control != FlowControl::None || raw_include {
                p.reset_flow_control();
            }

            p.set_sandbox(false);
            p.state.input_stack.remove(&canonic); // Collect stack
            Ok(chunk)
        } else {
            let formatted = format!(
                "File path : \"{}\" doesn't exist or not a file",
                file_path.display()
            );
            Err(RadError::InvalidExecution(formatted))
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("incread", AuthType::FIN, p)? {
            return Ok(None);
        }
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("if")
            .cursors_with_len(input)?;

        if !cursors.is_empty() {
            let path = cursors.get_text(p, 0)?;
            Ok(p.execute_macro(level, "incread", "include", &path)?)
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
        input: MacroInput,
        level: usize,
        p: &mut Processor,
    ) -> RadResult<Option<String>> {
        if !Utils::is_granted("tempin", AuthType::FIN, p)? {
            return Ok(None);
        }
        let file = p.get_temp_path().to_owned();
        let cursors = ArgParser::new()
            .level(level)
            .macro_name("tempin")
            .cursors_with_len(input)?;

        // TODO TT
        //
        // Change this to inline method
        let raw_include = cursors.get_bool(p, 0).unwrap_or(false);
        Self::include_inner(p, level, file, raw_include)
    }

    // Keyword macros end
    // ----------
}

// <MISC>

#[inline]
#[allow(clippy::too_many_arguments)]
fn append_captured_expressions(
    reg: &Regex,
    source: &str,
    container: &mut String,
    p: &mut Processor,
    level: usize,
    caller: &str,
    macro_name: &str,
    macro_arguments: &str,
) -> RadResult<bool> {
    let mut capture_occured = false;
    for cap in reg.captures_iter(source) {
        capture_occured = true;
        let mut cap = cap.iter().peekable();
        let total = cap.next(); // First capture group
        let captured = if cap.peek().is_some() {
            cap.filter(|s| s.is_some())
                .map(|s| s.unwrap().as_str())
                .join(" ")
        } else {
            total
                .filter(|s| s.is_some())
                .map(|s| s.unwrap().as_str())
                .unwrap()
                .to_string()
        };
        let expanded = p
            .execute_macro(
                level,
                caller,
                macro_name,
                &(macro_arguments.to_owned() + &captured),
            )?
            .unwrap_or_default();
        container.push_str(&expanded);
    }
    Ok(capture_occured)
}

// </MISC>
