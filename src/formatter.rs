//! Formatter for various formats

use crate::error::RadError;
use crate::RadResult;
use dcsv::VirtualArray;
use itertools::Itertools;
use std::fmt::Write;

// TODO TT
// use once_cell::sync::Lazy;
// static EMPTY_ERROR: Lazy<RadError> = Lazy::new(|| RadError::InvalidArgument(String::from("")));

/// Formatter that constructs multiple text formats
pub(crate) struct Formatter;

impl Formatter {
    /// Convert csv to corresponding format table
    ///
    /// Available formats are
    /// - github
    /// - wikitext
    /// - html
    pub(crate) fn csv_to_table(table_format: &str, data: &str, newline: &str) -> RadResult<String> {
        let data = dcsv::Reader::new()
            .trim(true)
            .ignore_empty_row(true)
            .has_header(false)
            .array_from_stream(data.as_bytes())?;
        if data.rows.is_empty() {
            return Err(RadError::InvalidArgument(
                "Table cannot be constructed from empty value".to_string(),
            ));
        }
        let table = match table_format {
            "github" => Formatter::gfm_table(&data, newline)?,
            "wikitext" => Formatter::wikitext_table(&data, newline)?,
            "html" => Formatter::html_table(&data, newline)?,
            _ => {
                return Err(RadError::InvalidConversion(format!(
                    "Unsupported table format : {}",
                    table_format
                )))
            }
        };
        Ok(table)
    }

    // ----------
    // Formatting methods start
    // <FORMAT>
    /// Execute sequence of macros from csv data
    pub fn csv_to_macros(
        macro_name: &str,
        mut macro_arguments: String,
        data: &str,
        newline: &str,
    ) -> RadResult<String> {
        let data = dcsv::Reader::new()
            .has_header(false)
            .array_from_stream(data.as_bytes())?;
        let mut exec = String::new();
        let mut iter = data.rows.iter().peekable();
        if !macro_arguments.is_empty() {
            macro_arguments.push(',');
        }
        while let Some(row) = iter.next() {
            write!(
                exec,
                "${}({}{})",
                macro_name,
                macro_arguments,
                row.iter().join(",")
            )?;
            if iter.peek().is_some() {
                exec.push_str(newline);
            }
        }
        Ok(exec)
    }

    /// Format csv into github formatted table
    fn gfm_table(data: &VirtualArray, newline: &str) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if header.is_none() {
            return Err(RadError::InvalidArgument(
                "Table cannot be constructed from empty value".to_string(),
            ));
        }
        let header = header.unwrap();
        table.push('|');
        let header_count = header.len();
        for h in header {
            write!(table, "{}|", h)?;
        }

        // Add separator
        write!(table, "{}|", newline)?;
        for _ in 0..header_count {
            table.push_str("-|");
        }

        for row in data_iter {
            write!(table, "{}|", newline)?;
            for value in row {
                write!(table, "{}|", value)?;
            }
        }

        Ok(table)
    }

    /// Format csv into wikitext formatted table
    fn wikitext_table(data: &VirtualArray, newline: &str) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if header.is_none() {
            return Err(RadError::InvalidArgument(
                "Table cannot be constructed from empty value".to_string(),
            ));
        }
        let header = header.unwrap();

        // Add header
        write!(table, "{{| class=\"wikitable\"{}", newline)?;
        // ! Header text !! Header text !! Header text
        // |-
        for h in header {
            write!(table, "!{}{}", h, newline)?;
        }
        // Header separator
        write!(table, "|-{}", newline)?;
        for row in data_iter {
            for value in row {
                write!(table, "|{}{}", value, newline)?;
            }
            write!(table, "|-{}", newline)?;
        }
        table.push_str("|}");
        Ok(table)
    }

    /// Format csv into html formatted table
    fn html_table(data: &VirtualArray, newline: &str) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if header.is_none() {
            return Err(RadError::InvalidArgument(
                "Table cannot be constructed from empty value".to_string(),
            ));
        }
        let header = header.unwrap();
        // Add header parts
        write!(table, "<table>{0}\t<thead>{0}\t\t<tr>{0}", newline)?;
        for h in header {
            write!(table, "\t\t\t<th>{}</th>{}", h, newline)?;
        }
        write!(table, "\t\t</tr>{0}\t</thead>{0}\t<tbody>{0}", newline)?;
        for row in data_iter {
            write!(table, "\t\t<tr>{}", newline)?;
            for value in row {
                write!(table, "\t\t\t<td>{}</td>", value)?;
            }
            write!(table, "\t\t</tr>{}", newline)?;
        }
        write!(table, "\t</tbody>{}</table>", newline)?;
        Ok(table)
    }
}
