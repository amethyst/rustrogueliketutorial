use rltk::prelude::*;
mod builder;
pub use builder::*;
mod logstore;
use logstore::*;
pub use logstore::{clear_log, log_display};

pub struct GameLog {
    pub entries : Vec<String>
}

pub struct LogFragment {
    pub color : RGB,
    pub text : String
}
