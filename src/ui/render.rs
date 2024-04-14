use crossterm::{ cursor, execute, queue, style::{Color, Print, ResetColor, SetForegroundColor}, terminal::{self, ClearType} };
use std::io::{self, Stdout, Write};
use crate::app::{App, Message};
use crate::commands;
use crate::text::{pad_to_width, truncate_to_width, visible_width, wrap_text};
use crate::ui::draw::{box_frame, draw_centered, draw_text, input_view};

pub fn intro_art() -> [&'static str; 6] {
    [
        r"  _____                  _____        _____ _      ___   ",
        r" |__  /___ _ __ ___     / ____|      / ____| |    |__ \  ",
        r"   / // _ \ '__/ __|   | |  __  __ _| |    | |       ) | ",
        r"  / /|  __/ |  \__ \   | | |_ |/ _` | |    | |      / /  ",
        r" /____\___|_|  |___/   | |__| | (_| | |____| |____ / /_  ",
        r"                         \_____|\__,_|\_____|______|____| ",
    ]
}
