use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::io;
mod table_parsing;
use table_parsing::TableSQL;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
/// Representa qual elemento possui o foco atual na interface.
#[derive(PartialEq)]
enum Focus {
    Input,
    ProcessBtn,
    ClearBtn,
}

/// Estado global da aplicação.
struct App {
    input: Input,
    table_data: TableSQL,
    focus: Focus,
    exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            input: Input::default(),
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

    /// Lida com os inputs do usuário via crossterm.
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            // Processa apenas quando a tecla é pressionada (ignora releases).
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => self.exit = true,
                    KeyCode::Tab => self.cycle_focus_forward(),
                    KeyCode::BackTab => self.cycle_focus_backward(), // Shift + Tab
                    KeyCode::Enter => self.handle_enter(),
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        // Se não estiver no modo de input de texto, 'q' sai da aplicação.
                        if self.focus != Focus::Input {
                            self.exit = true;
                        } else {
                            self.input.handle_event(&Event::Key(key));
                        }
                    }
                    _ => {
                        // Se o Input estiver focado, repassa o evento para o gerenciador de input.
                        if self.focus == Focus::Input {
                            self.input.handle_event(&Event::Key(key));
                        }
                    }
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
            Focus::Input => self.focus = Focus::ProcessBtn, // Enter no Input move para o botão
        }
    }

    /// Lógica de injeção de dados na tabela (Equivalente à on_button_pressed -> process).
    fn process_data(&mut self) {
        let text = self.input.value().to_string();

        self.table_data = table_parsing::parsing_input(&text);
    }

    /// Lógica de limpeza de estado (Equivalente à on_button_pressed -> clear).
    fn clear_data(&mut self) {
        self.input.reset();
        self.table_data = TableSQL::new();
        self.focus = Focus::Input;
    }

    /// Responsável pelo pipeline de UI no modo imediato.
    fn draw(&self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Length(3), // Input Block
                Constraint::Length(3), // Buttons Block
                Constraint::Min(5),    // Table Block
                Constraint::Length(1), // Footer
            ])
            .split(frame.area());

        // --- HEADER ---
        let header = Paragraph::new("TUI Text Processor")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(header, main_layout[0]);

        // --- INPUT TEXT ---
        let input_style = if self.focus == Focus::Input {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let input_widget = Paragraph::new(self.input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Digite o texto a ser formatado")
                .style(input_style),
        );
        frame.render_widget(input_widget, main_layout[1]);

        // Renderização manual do cursor de hardware se o Input estiver focado.
        if self.focus == Focus::Input {
            frame.set_cursor_position((
                main_layout[1].x + self.input.visual_cursor() as u16 + 1,
                main_layout[1].y + 1,
            ));
        }

        // --- BUTTONS ---
        let btn_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[2]);

        // Estilização dinâmica condicional baseada na máquina de estado do foco.
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
        let footer = Paragraph::new("Tab: Alternar Foco | Enter: Selecionar | Esc: Sair")
            .style(Style::default().fg(Color::DarkGray));
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
