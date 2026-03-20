use ratatui::{
    crossterm::{
        execute,
        event::{
            DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind,
            KeyModifiers, self as crossterm_event,
        },
    },
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::io;
use tui_textarea::TextArea;

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
struct App<'a> {
    /// O tui-textarea gerencia todo o estado do texto, cursor, scroll e histórico.
    textarea: TextArea<'a>,
    table_data: TableSQL,
    focus: Focus,
    exit: bool,
    /// Posição vertical de scroll da tabela (primeira linha visível)
    table_scroll: usize,
}

impl<'a> App<'a> {
    fn new() -> Self {
        let mut textarea = TextArea::default();
        // Configuração inicial do bloco de texto
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Digite a Query SQL (Ctrl+Enter para processar)"),
        );

        Self {
            textarea,
            table_data: TableSQL::new(),
            focus: Focus::Input,
            exit: false,
            table_scroll: 0,
        }
    }

    /// Loop principal de renderização e interceptação de eventos de hardware.
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            // Nota: passamos `self` como mutável para o draw, pois o tui-textarea
            // exige referência mutável para atualizar cursores e estilos de bloco dinamicamente.
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Obtém o texto completo unindo as linhas gerenciadas pelo tui-textarea
    fn get_full_input(&self) -> String {
        self.textarea.lines().join("\n")
    }

    /// Limpa o texto vindo da área de transferência.
    fn sanitize_paste(input: &str) -> String {
        input
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .chars()
            .filter(|&c| (c == '\n' || c == '\t' || !c.is_control()) && c != '\u{FEFF}')
            .collect()
    }

    /// Lida com os inputs do usuário via crossterm.
    fn handle_events(&mut self) -> io::Result<()> {
        let event = crossterm_event::read()?;

        match &event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match key.code {
                    KeyCode::Esc => {
                        self.exit = true;
                        return Ok(());
                    }
                    KeyCode::Tab => {
                        self.cycle_focus_forward();
                        return Ok(());
                    }
                    KeyCode::BackTab => {
                        self.cycle_focus_backward();
                        return Ok(());
                    }
                    // Atalho Global: Ctrl+Enter para processar dados
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.focus == Focus::Input {
                            self.process_data();
                        }
                        return Ok(());
                    }
                    // Enter para acionar botões
                    KeyCode::Enter if self.focus != Focus::Input => {
                        self.handle_enter();
                        return Ok(());
                    }
                    // 'q' para sair quando não estiver no input
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        if self.focus != Focus::Input
                            && !key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            self.exit = true;
                            return Ok(());
                        }
                    }
                    // Navegação vertical na tabela com Setas (apenas se houver linhas)
                    KeyCode::Up => {
                        if self.table_scroll > 0 {
                            self.table_scroll -= 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Down => {
                        let max_rows = self.table_data.rows.len();
                        // Limita o scroll para não ultrapassar o número de linhas
                        // Deve deixar espaço para a última linha ficar visível
                        if max_rows > 0 && self.table_scroll < max_rows.saturating_sub(1) {
                            self.table_scroll += 1;
                        }
                        return Ok(());
                    }
                    KeyCode::Home => {
                        self.table_scroll = 0;
                        return Ok(());
                    }
                    KeyCode::End => {
                        let max_rows = self.table_data.rows.len();
                        if max_rows > 0 {
                            self.table_scroll = max_rows.saturating_sub(1);
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            }
            Event::Paste(data) => {
                // Interceptamos a colagem manualmente, sanitizamos e injetamos no textarea
                if self.focus == Focus::Input {
                    let clean_data = Self::sanitize_paste(data);
                    self.textarea.insert_str(&clean_data);
                }
                return Ok(());
            }
            _ => {}
        }

        // Delegação de Eventos:
        // Se o evento não for um atalho global capturado acima e o foco estiver no input,
        // repassamos o evento bruto para o tui-textarea. Ele cuidará automaticamente das setas,
        // backspace, delete, digitação normal, navegação por palavras (Ctrl+Left/Right), etc.
        if self.focus == Focus::Input {
            self.textarea.input(event);
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
            Focus::Input => {}
        }
    }

    /// Gera um indicador visual de scroll (▲▼ ou similar)
    fn scroll_indicator(current: usize, total: usize) -> String {
        if total == 0 {
            return String::new();
        }
        
        let percentage = (current as f32 / total as f32 * 100.0) as u32;
        
        if current == 0 {
            "▼".to_string()
        } else if current == total - 1 {
            "▲".to_string()
        } else {
            format!("{}%", percentage)
        }
    }

    fn process_data(&mut self) {
        let text = self.get_full_input();
        self.table_data = table_parsing::parsing_input(&text);
    }

    fn clear_data(&mut self) {
        // Reinicializa o textarea do zero e reaplica as configurações base
        self.textarea = TextArea::default();
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Digite a Query SQL (Ctrl+Enter para processar)"),
        );
        self.table_data = TableSQL::new();
        self.focus = Focus::Input;
        self.table_scroll = 0;
    }

    /// Responsável pelo pipeline de UI no modo imediato.
    fn draw(&mut self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(5),    // Input Block
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

        // --- INPUT TEXTAREA ---
        // Atualiza a aparência do textarea com base no foco atual
        if self.focus == Focus::Input {
            self.textarea.set_style(Style::default().fg(Color::Yellow));
            self.textarea
                .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
            self.textarea
                .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        } else {
            self.textarea
                .set_style(Style::default().fg(Color::DarkGray));
            self.textarea
                .set_cursor_style(Style::default().add_modifier(Modifier::HIDDEN));
            self.textarea.set_cursor_line_style(Style::default()); // Remove underline
        }

        // O tui-textarea implementa Widget nativamente
        frame.render_widget(&self.textarea, main_layout[1]);

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
        
        // Calcula quantas linhas podem ser visualizadas na área da tabela
        // Reserva espaço para header (1) e border (1) = 2 linhas
        let available_height = main_layout[3].height.saturating_sub(2) as usize;
        
        // Aplica scroll: pega apenas as linhas visíveis a partir de table_scroll
        let visible_rows: Vec<Row> = rows
            .into_iter()
            .skip(self.table_scroll)
            .take(available_height)
            .collect();

        let table = Table::new(
            visible_rows,
            [
                Constraint::Percentage(33),
                Constraint::Percentage(50),
                Constraint::Percentage(17),
            ],
        )
        .header(table_header)
        .block(
            Block::bordered()
                .title(format!(
                    "Output (linha {}/{}) {}",
                    self.table_scroll + 1,
                    self.table_data.rows.len().max(1),
                    Self::scroll_indicator(self.table_scroll, self.table_data.rows.len())
                ))
                .border_style(Style::default().fg(Color::Green)),
        );
        frame.render_widget(table, main_layout[3]);

        // --- FOOTER ---
        let footer_text = vec![
            Line::raw("Tab: Alternar Foco | Ctrl+Enter: Processar | Esc: Sair | ↑/↓: Scroll Tabela | Home/End: Ir para início/fim"),
            Line::raw(
                "Atalhos do Editor: Setas, Home/End, Backspace/Delete, Ctrl+W, Ctrl+A, Ctrl+E suportados nativamente.",
            ),
        ];
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Left);
        frame.render_widget(footer, main_layout[4]);
    }
}

fn main() -> io::Result<()> {
    execute!(io::stdout(), EnableBracketedPaste)?;

    let mut terminal = ratatui::init();
    let mut app = App::new();
    let app_result = app.run(&mut terminal);

    ratatui::restore();
    execute!(io::stdout(), DisableBracketedPaste)?;

    app_result
}
