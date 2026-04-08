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
    pub cols_len: usize,
    pub values_len: usize,
}

impl TableSQL {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            rows: Vec::new(),
            cols_len: 0,
            values_len: 0,
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


pub fn parsing_input(input: &str) -> Result<TableSQL, String> {
    let mut table = TableSQL::new();

    // Valida se o input não está vazio
    if input.trim().is_empty() {
        return Err("A query SQL está vazia. Digite uma query INSERT válida.".to_string());
    }

    // // Normaliza o input antes de fazer parsing
    // let normalized_input = normalize_sql_input(input);

    // Parser com dialeto genérico (compatível com ANSI SQL)
    let dialect = GenericDialect {};
    let parser = Parser::new(&dialect).try_with_sql(input);

    let mut parser = parser.map_err(|e| {
        format!("Erro ao criar parser SQL: {}", e)
    })?;

    let statements = parser.parse_statements().map_err(|e| {
            format!("Erro ao fazer parsing da query SQL: {}. Verifique a sintaxe.", e)
    })?;

    // Procura por um statement INSERT
    let mut found_insert = false;
    for statement in statements {
        if let Statement::Insert {
            table_name,
            columns,
            source,
            ..
        } = statement
        {
            found_insert = true;
            table.name = table_name.to_string();

            // Extrai os valores do INSERT
            if let Some(insert_source) = source {
                // O source é uma SELECT statement que contém os VALUES
                if let sqlparser::ast::SetExpr::Values(values) = *insert_source.body {
                    if !values.rows.is_empty() {
                        let row_values = &values.rows[0];
                        table.cols_len = columns.len();
                        table.values_len = row_values.len();

                        // Valida se o número de colunas bate com o número de valores
                        if columns.len() != row_values.len() {
                            return Err(format!(
                                "Quantidade de colunas ({}) não bate com quantidade de valores ({})",
                                columns.len(),
                                row_values.len()
                            ));
                        }

                        // Mapeia colunas com valores
                        for (i, col) in columns.iter().enumerate() {
                            if i < table.values_len {
                                let col_name = col.to_string();
                                let col_value = extract_value(&row_values[i]);

                                table.rows.push(TableRow {
                                    col_name,
                                    col_value,
                                });
                            }
                        }
                    } else {
                        return Err("Nenhuma linha de valores foi encontrada no INSERT.".to_string());
                    }
                } else {
                    return Err("Formato de VALUES inválido. Use INSERT INTO ... VALUES (...)".to_string());
                }
            } else {
                return Err("INSERT sem cláusula VALUES não é suportado.".to_string());
            }

            break;
        }
    }

    if !found_insert {
        return Err("Nenhum statement INSERT foi encontrado. Digite uma query INSERT INTO válida.".to_string());
    }

    Ok(table)
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
