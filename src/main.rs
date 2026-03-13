use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use std::io;

mod table_parsing;
use table_parsing::TableSQL;

/// Representa qual elemento possui o foco atual na interface.
#[derive(PartialEq)]
enum Focus {
    Input,
    ProcessBtn,
    ClearBtn,
}

/// Estado global da aplicação.
struct App {
    /// Armazena as linhas de input do usuário
    input_lines: Vec<String>,
    /// Índice da linha atual de edição
    current_line: usize,
    /// Posição do cursor na linha atual (em caracteres, não bytes)
    cursor_pos: usize,
    table_data: TableSQL,
    focus: Focus,
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            input_lines: vec![String::new()],
            current_line: 0,
            cursor_pos: 0,
            table_data: TableSQL::new(),
            focus: Focus::Input,
            exit: false,
        }
    }

    /// Loop principal de renderização e interceptação de eventos de hardware.
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Obtém o texto completo de todas as linhas
    fn get_full_input(&self) -> String {
        self.input_lines.join("\n")
    }

    /// Insere um caractere na posição atual do cursor (respeitando UTF-8)
    fn insert_char(&mut self, ch: char) {
        let current_line = self.current_line;
        let cursor_pos = self.cursor_pos;

        let line = &mut self.input_lines[current_line];
        let byte_pos = Self::char_pos_to_byte_pos_in_line(line, cursor_pos);
        line.insert(byte_pos, ch);
        self.cursor_pos += 1;
    }

    /// Remove o caractere anterior ao cursor (backspace)
    fn backspace(&mut self) {
        let current_line = self.current_line;
        let cursor_pos = self.cursor_pos;

        if cursor_pos > 0 {
            let line = &mut self.input_lines[current_line];
            let byte_pos = Self::char_pos_to_byte_pos_in_line(line, cursor_pos - 1);
            let char_len = line[byte_pos..].chars().next().unwrap().len_utf8();
            line.drain(byte_pos..byte_pos + char_len);
            self.cursor_pos -= 1;
        } else if current_line > 0 {
            // Se estamos no início da linha, junta com a linha anterior
            let current = self.input_lines.remove(current_line);
            self.current_line -= 1;
            self.cursor_pos = self.input_lines[self.current_line].chars().count();
            self.input_lines[self.current_line].push_str(&current);
        }
    }

    /// Remove o caractere após o cursor (delete)
    fn delete(&mut self) {
        let current_line = self.current_line;
        let cursor_pos = self.cursor_pos;
        let total_lines = self.input_lines.len();

        let char_count = self.input_lines[current_line].chars().count();
        if cursor_pos < char_count {
            let line = &mut self.input_lines[current_line];
            let byte_pos = Self::char_pos_to_byte_pos_in_line(line, cursor_pos);
            let char_len = line[byte_pos..].chars().next().unwrap().len_utf8();
            line.drain(byte_pos..byte_pos + char_len);
        } else if current_line < total_lines - 1 {
            // Se estamos no final da linha, junta com a próxima
            let next = self.input_lines.remove(current_line + 1);
            self.input_lines[current_line].push_str(&next);
        }
    }

    /// Cria uma nova linha a partir da posição atual do cursor
    fn split_line(&mut self) {
        let line = &mut self.input_lines[self.current_line];
        let rest = line.split_off(self.cursor_pos);
        self.input_lines.insert(self.current_line + 1, rest);
        self.current_line += 1;
        self.cursor_pos = 0;
    }

    /// Converte posição de caractere para posição de byte (UTF-8)
    fn char_pos_to_byte_pos_in_line(line: &str, char_pos: usize) -> usize {
        line.chars()
            .take(char_pos)
            .map(|c| c.len_utf8())
            .sum()
    }

    /// Move o cursor para a esquerda
    fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        } else if self.current_line > 0 {
            self.current_line -= 1;
            self.cursor_pos = self.input_lines[self.current_line].chars().count();
        }
    }

    /// Move o cursor para a direita
    fn move_cursor_right(&mut self) {
        let line_char_count = self.input_lines[self.current_line].chars().count();
        if self.cursor_pos < line_char_count {
            self.cursor_pos += 1;
        } else if self.current_line < self.input_lines.len() - 1 {
            self.current_line += 1;
            self.cursor_pos = 0;
        }
    }

    /// Move o cursor para o início da linha
    fn move_cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    /// Move o cursor para o final da linha
    fn move_cursor_end(&mut self) {
        self.cursor_pos = self.input_lines[self.current_line].chars().count();
    }

    /// Lida com os inputs do usuário via crossterm.
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            // Processa apenas quando a tecla é pressionada (ignora releases).
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => self.exit = true,
                    KeyCode::Tab => {
                        if self.focus == Focus::Input {
                            self.cycle_focus_forward();
                        } else {
                            self.cycle_focus_forward();
                        }
                    }
                    KeyCode::BackTab => self.cycle_focus_backward(),
                    KeyCode::Enter => {
                        if self.focus == Focus::Input {
                            // Ctrl+Enter para processar, Enter simples para nova linha
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                self.process_data();
                            } else {
                                self.split_line();
                            }
                        } else {
                            self.handle_enter();
                        }
                    }
                    KeyCode::Backspace => {
                        if self.focus == Focus::Input {
                            self.backspace();
                        }
                    }
                    KeyCode::Delete => {
                        if self.focus == Focus::Input {
                            self.delete();
                        }
                    }
                    KeyCode::Left => {
                        if self.focus == Focus::Input {
                            self.move_cursor_left();
                        }
                    }
                    KeyCode::Right => {
                        if self.focus == Focus::Input {
                            self.move_cursor_right();
                        }
                    }
                    KeyCode::Home => {
                        if self.focus == Focus::Input {
                            self.move_cursor_home();
                        }
                    }
                    KeyCode::End => {
                        if self.focus == Focus::Input {
                            self.move_cursor_end();
                        }
                    }
                    KeyCode::Char(ch) => {
                        if self.focus == Focus::Input {
                            // Ignora Ctrl+C para não interferir com o terminal
                            if !(key.modifiers.contains(KeyModifiers::CONTROL)
                                && (ch == 'c' || ch == 'C'))
                            {
                                self.insert_char(ch);
                            }
                        } else if (ch == 'q' || ch == 'Q')
                            && !key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            self.exit = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn cycle_focus_forward(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::ProcessBtn,
            Focus::ProcessBtn => Focus::ClearBtn,
            Focus::ClearBtn => Focus::Input,
        };
    }

    fn cycle_focus_backward(&mut self) {
        self.focus = match self.focus {
            Focus::Input => Focus::ClearBtn,
            Focus::ProcessBtn => Focus::Input,
            Focus::ClearBtn => Focus::ProcessBtn,
        };
    }

    fn handle_enter(&mut self) {
        match self.focus {
            Focus::ProcessBtn => self.process_data(),
            Focus::ClearBtn => self.clear_data(),
            Focus::Input => {} // Já tratado em handle_events
        }
    }

    /// Lógica de injeção de dados na tabela
    fn process_data(&mut self) {
        let text = self.get_full_input();
        self.table_data = table_parsing::parsing_input(&text);
    }

    /// Lógica de limpeza de estado
    fn clear_data(&mut self) {
        self.input_lines = vec![String::new()];
        self.current_line = 0;
        self.cursor_pos = 0;
        self.table_data = TableSQL::new();
        self.focus = Focus::Input;
    }

    /// Responsável pelo pipeline de UI no modo imediato.
    fn draw(&self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(5),    // Input Block (maior para múltiplas linhas)
                Constraint::Length(3), // Buttons Block
                Constraint::Min(5),    // Table Block
                Constraint::Length(2), // Footer
            ])
            .split(frame.area());

        // --- HEADER ---
        let header = Paragraph::new("TUI Text Processor - SQL INSERT Formatter")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(header, main_layout[0]);

        // --- INPUT TEXT (MULTILINE) ---
        let input_style = if self.focus == Focus::Input {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let input_text: Vec<Line> = self
            .input_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                if idx == self.current_line {
                    // Destaca a linha atual com o cursor
                    let mut content = line.clone();
                    // Insere um marcador de cursor visualmente
                    let char_count = content.chars().count();
                    if self.cursor_pos <= char_count {
                        let byte_pos = Self::char_pos_to_byte_pos_in_line(&content, self.cursor_pos);
                        content.insert_str(byte_pos, "│");
                    } else {
                        content.push('│');
                    }
                    Line::raw(content)
                } else {
                    Line::raw(line.clone())
                }
            })
            .collect();

        let input_widget = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Digite a Query SQL (Ctrl+Enter para processar, Enter para nova linha)")
                    .style(input_style),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(input_widget, main_layout[1]);

        // --- BUTTONS ---
        let btn_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[2]);

        let process_style = if self.focus == Focus::ProcessBtn {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::Green)
        };
        let process_btn = Paragraph::new("Processar")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).style(process_style));
        frame.render_widget(process_btn, btn_layout[0]);

        let clear_style = if self.focus == Focus::ClearBtn {
            Style::default().fg(Color::Black).bg(Color::Red)
        } else {
            Style::default().fg(Color::Red)
        };
        let clear_btn = Paragraph::new("Limpar")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).style(clear_style));
        frame.render_widget(clear_btn, btn_layout[1]);

        // --- DATA TABLE ---
        let header_cells = ["Colunas", "Valores"].iter().map(|h| {
            Cell::from(Text::from(*h).alignment(Alignment::Left))
                .style(Style::default().add_modifier(Modifier::BOLD))
        });
        let table_header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = self.table_data.to_ratatui_rows();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(50),
                Constraint::Percentage(17),
            ],
        )
        .header(table_header)
        .block(
            Block::bordered()
                .title("Output")
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(table, main_layout[3]);

        // --- FOOTER ---
        let footer_text = vec![
            Line::raw("Tab: Alternar Foco | Ctrl+Enter: Processar | Enter: Nova Linha | Esc: Sair"),
            Line::raw("↑↓←→: Mover Cursor | Home/End: Início/Fim da Linha | Backspace/Delete: Deletar"),
        ];
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left);
        frame.render_widget(footer, main_layout[4]);
    }
}

fn main() -> io::Result<()> {
    // Inicialização da infraestrutura do terminal via crossterm em modo raw.
    let mut terminal = ratatui::init();

    // Instanciação e ciclo de vida da aplicação
    let mut app = App::new();
    let app_result = app.run(&mut terminal);

    // Limpeza mandatória do buffer alternativo e restauro do terminal independente do resultado da app.
    ratatui::restore();

    app_result
}
