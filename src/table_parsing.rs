use ratatui::{
    style::{Modifier, Style},
    text::Text,
    widgets::{Cell, Row},
};
use regex::Regex;

/// Um par coluna → valor que representa uma linha da tabela na UI.
pub struct TableRow {
    pub col_name: String,
    pub col_value: String,
}

/// Dados parseados de um INSERT SQL.
pub struct TableSQL {
    pub name: String,
    pub rows: Vec<TableRow>,
}

impl TableSQL {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            rows: Vec::new(),
        }
    }

    /// Converte os dados internos em `Row`s prontos para o widget `Table` do ratatui.
    pub fn to_ratatui_rows(&self) -> Vec<Row> {
        self.rows
            .iter()
            .map(|r| {
                Row::new([
                    Cell::from(Text::raw(r.col_name.as_str())),
                    Cell::from(Text::raw(r.col_value.as_str())),
                ])
                .height(1)
            })
            .collect()
    }
}

pub fn parsing_input(input: &str) -> TableSQL {
    let mut table = TableSQL::new();
    let re = Regex::new(
        r"(?i)^\s*INSERT\s+into\s+(?P<table_name>[\w.]+)\s*\((?P<cols_names>[\w,\s]+)\)\s+VALUES\s*\((?P<cols_values>[\s\S]+)\)\s*;\s*$",
    )
    .unwrap();

    let Some(caps) = re.captures(input) else {
        return table;
    };

    table.name = caps["table_name"].to_string();

    let names: Vec<&str> = caps["cols_names"].split(',').map(str::trim).collect();
    let values: Vec<&str> = caps["cols_values"].split(',').map(str::trim).collect();

    table.rows = names
        .into_iter()
        .zip(values)
        .map(|(name, value)| TableRow {
            col_name: name.to_string(),
            col_value: value.to_string(),
        })
        .collect();

    table
}
