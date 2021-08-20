use lazy_static::lazy_static;
use regex::Regex;
use crate::consts::ESCAPED_COMMA;
use crate::error::RadError;

lazy_static!{
   pub static ref ESCAPE: Regex = Regex::new(r"\\,").unwrap();
   pub static ref RESTORE: Regex = Regex::new(r"@COMMA@").unwrap();
}

pub struct Formatter;

impl Formatter {
    pub fn csv_to_table(table_format : &str, data: &str, newline: &str) -> Result<String, RadError> {
        let data = Self::escape_comma(data);
        let mut reader = 
            csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(data.as_bytes());
        let table = match table_format {
            "github" => Formatter::gfm_table(&mut reader, newline)?,
            "wikitext" => Formatter::wikitext_table(&mut reader, newline)?,
            _ => return Err(RadError::UnsupportedTableFormat(format!("Unsupported table format : {}", table_format)))
        };
        let table = Self::restore_comma(&table);
        Ok(table)
    }

    pub fn csv_to_macros(macro_name: &str, data: &str, newline: &str) 
        -> Result<String, RadError> 
    {
        let data = Self::escape_comma(data);
        let mut exec = String::new();
        let mut reader = 
            csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(data.as_bytes());
        let mut row_iter = reader.records().peekable();
        while let Some(row) = row_iter.next() {
            exec.push_str(&format!("${}(", macro_name));
            let record = row?;
            let mut col_iter = record.iter().peekable();
            // Column iteration
            while let Some(value) = col_iter.next() {
                exec.push_str(value);
                if let Some(_) = col_iter.peek() {
                    exec.push(',');
                }
            }
            exec.push(')');
            if let Some(_) = row_iter.peek() {
                exec.push_str(newline);
            }
        }

        let exec = Self::restore_comma(&exec);
        Ok(exec)
    }

    fn gfm_table(reader : &mut csv::Reader<&[u8]>, newline: &str) -> Result<String, RadError> {
        let mut table = String::new();
        table.push('|');
        let header_iter = reader.headers()?;
        let header_count = header_iter.len();
        for header in header_iter {
            table.push_str(header);
            table.push('|');
        }
        // Add separator
        table.push_str(newline);
        table.push('|');
        for _ in 0..header_count {
            table.push_str("-|");
        }
        for record in reader.records() {
            table.push_str(newline);
            table.push('|');
            for column in record?.iter() {
                table.push_str(column);
                table.push('|');
            }
        }

        Ok(table)
    }

    fn wikitext_table(reader : &mut csv::Reader<&[u8]>, newline: &str) -> Result<String, RadError> {
        let mut table = String::new();
        // Add header
        table.push_str("{| class=\"wikitable\"");
        table.push_str(newline);
        // ! Header text !! Header text !! Header text
        // |-
        let header_iter = reader.headers()?;
        for header in header_iter {
            table.push('!');
            table.push_str(header);
            table.push_str(newline);
        }
        // Header separator
        table.push_str("|-"); 
        table.push_str(newline); 
        for record in reader.records() {
            for column in record?.iter() {
                table.push('|');
                table.push_str(column);
                table.push_str(newline); 
            }
            table.push_str("|-"); 
            table.push_str(newline); 
        }
        table.push_str("|}");
        Ok(table)
    }

    fn escape_comma(source : &str) -> String {
        ESCAPE.replace_all(source,ESCAPED_COMMA).to_string()
    }

    fn restore_comma(source : &str) -> String {
        RESTORE.replace_all(source,",").to_string()
    }
}
