use ratatui::{
    text::Text,
    widgets::{Cell, Row},
};
use sqlparser::ast::{Expr, Statement};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

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
    pub fn to_ratatui_rows(&self) -> Vec<Row<'_>> {
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

    // Parser com dialeto genérico (compatível com ANSI SQL)
    let dialect = GenericDialect {};
    let parser = Parser::new(&dialect).try_with_sql(input);

    let Ok(mut parser) = parser else {
        return table;
    };

    let Ok(statements) = parser.parse_statements() else {
        return table;
    };

    // Procura por um statement INSERT
    for statement in statements {
        if let Statement::Insert {
            table_name,
            columns,
            source,
            ..
        } = statement
        {
            table.name = table_name.to_string();

            // Extrai os valores do INSERT
            if let Some(insert_source) = source {
                // O source é uma SELECT statement que contém os VALUES
                if let sqlparser::ast::SetExpr::Values(values) = *insert_source.body {
                    if !values.rows.is_empty() {
                        let row_values = &values.rows[0];

                        // Mapeia colunas com valores
                        for (i, col) in columns.iter().enumerate() {
                            if i < row_values.len() {
                                let col_name = col.to_string();
                                let col_value = extract_value(&row_values[i]);

                                table.rows.push(TableRow {
                                    col_name,
                                    col_value,
                                });
                            }
                        }
                    }
                }
            }

            break;
        }
    }

    table
}

/// Extrai o valor de uma Expr como String
fn extract_value(expr: &Expr) -> String {
    match expr {
        Expr::Value(value) => {
            use sqlparser::ast::Value;
            match value {
                Value::SingleQuotedString(s) => s.clone(),
                Value::Number(n, _) => n.clone(),
                Value::Boolean(b) => b.to_string(),
                Value::Null => "NULL".to_string(),
                _ => expr.to_string(),
            }
        }
        _ => expr.to_string(),
    }
}