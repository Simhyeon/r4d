use dcsv::ReadOnlyDataRef;

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
            .has_header(false)
            .read_from_stream(data.as_bytes())?;
        let data_ref = data.read_only_ref();
        if data_ref.rows.len() == 0 {
            return Err(RadError::InvalidArgument(
                "Table cannot be constructed from empty value".to_string(),
            ));
        }
        let table = match table_format {
            "github" => Formatter::gfm_table(&data_ref, newline)?,
            "wikitext" => Formatter::wikitext_table(&data_ref, newline)?,
            "html" => Formatter::html_table(&data_ref)?,
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
            .read_from_stream(data.as_bytes())?;
        let mut exec = String::new();
        let mut iter = data.rows.iter().peekable();
        while let Some(row) = iter.next() {
            let row_str = row.to_string(&data.columns)?;
            exec.push_str(&format!("${}({})", macro_name, row_str));
            if let Some(_) = iter.peek() {
                exec.push_str(newline);
            }
        }
        Ok(exec)
    }

    // Format csv into github formatted table
    fn gfm_table(data: &ReadOnlyDataRef, newline: &str) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if let None = header {
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

        while let Some(row) = data_iter.next() {
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
    fn wikitext_table(data: &ReadOnlyDataRef, newline: &str) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if let None = header {
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
        while let Some(row) = data_iter.next() {
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
    fn html_table(data: &ReadOnlyDataRef) -> RadResult<String> {
        let mut table = String::new();
        let mut data_iter = data.rows.iter();
        let header = data_iter.next();
        if let None = header {
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
        while let Some(row) = data_iter.next() {
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
