import os
import customtkinter as ctk
from src.logic import CalculatorLogic


class CalculatorApp:
    def __init__(self, root):
        self.root = root
        self.root.title("Calculator")
        self.root.geometry("400x600")
        self.root.resizable(False, False)

        icon_path = os.path.join(os.path.dirname(__file__), "../assets/icon.ico")
        self.root.iconbitmap(icon_path)

        self.logic = CalculatorLogic()

        self.display_var = ctk.StringVar(value="0")
        self.display = ctk.CTkLabel(
            root,
            textvariable=self.display_var,
            font=("Helvetica", 36),
            anchor="e",
            width=380,
            height=100,
            corner_radius=10,
            fg_color=("white", "gray20"),
            text_color=("black", "white")
        )
        self.display.pack(pady=20, padx=10)

        self.buttons_frame = ctk.CTkFrame(root, corner_radius=10)
        self.buttons_frame.pack(pady=10, padx=10, fill="both", expand=True)

        self.create_buttons()

    def create_buttons(self):
        button_texts = [
            ["C", "DEL", "%", "/"],
            ["7", "8", "9", "*"],
            ["4", "5", "6", "-"],
            ["1", "2", "3", "+"],
            ["±", "0", ".", "="]
        ]

        operator_color = ("#FF9500", "#FF9500")
        button_color = ("#333333", "#333333")
        utility_color = ("#505050", "#505050")

        for i, row in enumerate(button_texts):
            for j, text in enumerate(row):
                if text in ["+", "-", "*", "/", "="]:
                    button = ctk.CTkButton(
                        self.buttons_frame,
                        text=text,
                        command=lambda t=text: self.on_button_click(t),
                        font=("Helvetica", 24),
                        corner_radius=10,
                        fg_color=operator_color,
                        text_color="white"
                    )
                elif text in ["C", "DEL", "%", "±"]:
                    button = ctk.CTkButton(
                        self.buttons_frame,
                        text=text,
                        command=lambda t=text: self.on_utility_click(t),
                        font=("Helvetica", 24),
                        corner_radius=10,
                        fg_color=utility_color,
                        text_color="white"
                    )
                else:
                    button = ctk.CTkButton(
                        self.buttons_frame,
                        text=text,
                        command=lambda t=text: self.on_button_click(t),
                        font=("Helvetica", 24),
                        corner_radius=10,
                        fg_color=button_color,
                        text_color="white"
                    )

                button.grid(row=i, column=j, sticky="nsew", padx=5, pady=5)

        for i in range(5):
            self.buttons_frame.grid_rowconfigure(i, weight=1)
            self.buttons_frame.grid_columnconfigure(i, weight=1)

    def on_button_click(self, button_text):
        if button_text == "=":
            result = self.logic.calculate(self.display_var.get())
            self.display_var.set(self.format_display(result))
        else:
            current_text = self.display_var.get()
            if current_text == "0" and button_text not in ["+", "-", "*", "/"]:
                self.display_var.set(button_text)
            else:
                new_text = current_text + button_text
                self.display_var.set(self.format_display(new_text))

    def on_utility_click(self, utility_text):
        current_text = self.display_var.get()
        if utility_text == "C":
            self.display_var.set("0")
        elif utility_text == "DEL":
            if len(current_text) > 1:
                self.display_var.set(current_text[:-1])
            else:
                self.display_var.set("0")
        elif utility_text == "%":
            try:
                result = str(float(current_text) / 100)
                self.display_var.set(self.format_display(result))
            except ValueError:
                self.display_var.set("Error")
        elif utility_text == "±":
            if current_text.startswith("-"):
                self.display_var.set(current_text[1:])
            else:
                self.display_var.set("-" + current_text)

    def format_display(self, value):
        """
        Formats the display value to prevent overflow.
        Uses scientific notation if the value exceeds 18 characters.
        """
        if len(value) > 18:
            try:
                formatted_value = f"{float(value):.10e}"
            except ValueError:
                formatted_value = "Error"
            return formatted_value
        return value


def start_app(root):
    app = CalculatorApp(root)
