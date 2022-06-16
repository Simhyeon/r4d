use dcsv::VirtualArray;

use crate::error::RadError;
use crate::RadResult;

/// Formatter that constructs multiple text formats
pub(crate) struct Formatter;

impl Formatter {
    /// Convert csv to corresponding format table
    ///
    /// Available formats are
    /// - github
    /// - wikitext
    /// - html
    pub fn csv_to_table(table_format: &str, data: &str, newline: &str) -> RadResult<String> {
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
            "html" => Formatter::html_table(&data)?,
            _ => {
                return Err(RadError::UnsupportedTableFormat(format!(
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
    pub fn csv_to_macros(macro_name: &str, data: &str, newline: &str) -> RadResult<String> {
        let data = dcsv::Reader::new()
            .has_header(false)
            .array_from_stream(data.as_bytes())?;
        let mut exec = String::new();
        let mut iter = data.rows.iter().peekable();
        while let Some(row) = iter.next() {
            exec.push_str(&format!("${}({})", macro_name, row.join(",")));
            if iter.peek().is_some() {
                exec.push_str(newline);
            }
        }
        Ok(exec)
    }

    // Format csv into github formatted table
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
            table.push_str(&h.to_string());
            table.push('|');
        }

        // Add separator
        table.push_str(newline);
        table.push('|');
        for _ in 0..header_count {
            table.push_str("-|");
        }

        for row in data_iter {
            table.push_str(newline);
            table.push('|');
            for value in row {
                table.push_str(&value.to_string());
                table.push('|');
            }
        }

        Ok(table)
    }

    // Format csv into wikitext formatted table
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
        table.push_str("{| class=\"wikitable\"");
        table.push_str(newline);
        // ! Header text !! Header text !! Header text
        // |-
        for h in header {
            table.push('!');
            table.push_str(&h.to_string());
            table.push_str(newline);
        }
        // Header separator
        table.push_str("|-");
        table.push_str(newline);
        for row in data_iter {
            for value in row {
                table.push('|');
                table.push_str(&value.to_string());
                table.push_str(newline);
            }
            table.push_str("|-");
            table.push_str(newline);
        }
        table.push_str("|}");
        Ok(table)
    }

    // Format csv into html formatted table
    fn html_table(data: &VirtualArray) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if header.is_none() {
            return Err(RadError::InvalidArgument(
                "Table cannot be constructed from empty value".to_string(),
            ));
        }
        let header = header.unwrap();
        table.push_str("<table>\n");
        // Add header
        table.push_str("\t<thead>\n\t\t<tr>\n");
        for h in header {
            table.push_str(&format!("\t\t\t<th>{}</th>\n", h));
        }
        table.push_str("\t\t</tr>\n\t</thead>\n");
        table.push_str("\t<tbody>\n");
        for row in data_iter {
            table.push_str("\t\t<tr>\n");
            for value in row {
                table.push_str(&format!("\t\t\t<td>{}</td>\n", value));
            }
            table.push_str("\t\t</tr>\n");
        }
        table.push_str("\t</tbody>\n</table>");
        Ok(table)
    }
}
