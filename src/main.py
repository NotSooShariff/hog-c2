# src/main.py

import os
import sys
import customtkinter as ctk
from src.gui import start_app

def resource_path(relative_path):
    try:
        base_path = sys._MEIPASS
    except AttributeError:
        base_path = os.path.abspath(".")

    return os.path.join(base_path, relative_path)

if __name__ == "__main__":
    root = ctk.CTk()

    icon_path = resource_path("assets/icon.ico")
    root.iconbitmap(icon_path)

    start_app(root)
    root.mainloop()
