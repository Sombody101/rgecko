use crate::argparse::parser::ColorMode;
use std::env;
use std::io::{self, IsTerminal};

pub fn get_color_support() -> ColorMode {
    return if ansi_supported() {
        ColorMode::Color256
    } else {
        ColorMode::NoColor
    };
}

fn ansi_supported() -> bool {
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }

    if !io::stdout().is_terminal() {
        return false;
    }

    if env::var_os("COLORTERM").is_some() {
        return true;
    }

    if let Ok(term) = env::var("TERM") {
        let term = term.to_lowercase();
        if term == "dumb" {
            return false;
        }

        if term.contains("color")
            || term.contains("256")
            || term.contains("xterm")
            || term.contains("ansi")
        {
            return true;
        }
    }

    if cfg!(windows) && env::var_os("WT_SESSION").is_some() {
        return true;
    }

    return false;
}
