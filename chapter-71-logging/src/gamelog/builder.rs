use rltk::prelude::*;
use super::{LogFragment, append_entry};

pub struct Logger {
    current_color : RGB,
    fragments : Vec<LogFragment>
}

impl Logger {
    pub fn new() -> Self {
        Logger{
            current_color : RGB::named(rltk::WHITE),
            fragments : Vec::new()
        }
    }

    pub fn color(mut self, color: (u8, u8, u8)) -> Self {
        self.current_color = RGB::named(color);
        self
    }

    pub fn append<T: ToString>(mut self, text : T) -> Self {
        self.fragments.push(
            LogFragment{
                color : self.current_color,
                text : text.to_string()
            }
        );
        self
    }

    pub fn log(self) {
        append_entry(self.fragments)
    }
}