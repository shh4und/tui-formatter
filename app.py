from textual.app import App, ComposeResult
from textual.containers import Container, Grid
from textual.widgets import Button, DataTable, Footer, Header, Input


class TextProcessorApp(App):
    CSS = """
    Screen {
        align: center middle;
    }

    Grid {
        grid-size: 2 3;
        grid-rows: auto auto 1fr;
        width: 100%;
        height: 100%;
        border: solid white 50%;
        padding: 0;
        margin: 0;
    }

    Input {
        column-span: 2;
    }

    Button {
        width: 100%;
    }

    #output {
        column-span: 2;
        border: solid green;
        padding: 0;
        height: 100%;
        width: 100%;
    }
    """

    BINDINGS = [("q", "quit", "Sair")]

    def compose(self) -> ComposeResult:
        yield Header()
        with Grid():
            yield Input(placeholder="Digite o texto a ser formatado", id="input")
            yield Button("Processar", variant="success", id="process")
            yield Button("Limpar", variant="warning", id="clear")
            yield DataTable(id="output")
        yield Footer()

    def on_mount(self) -> None:
        pass

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "process":
            input_widget = self.query_one("#input", Input)
            table = self.query_one("#output", DataTable)
            text = input_widget.value

            table.clear()
            table.add_columns("Colunas", "Valores", "Info")
            table.add_row(text, f"Processado: {len(text)} chars", "Status: OK")
            table.add_row("Linha 2", "Dados 2", "Info 2")
            table.add_row("Linha 3", "Dados 3", "Info 3")

        elif event.button.id == "clear":
            self.query_one("#input", Input).value = ""
            table = self.query_one("#output", DataTable)
            table.clear()


if __name__ == "__main__":
    app = TextProcessorApp()
    app.run()
