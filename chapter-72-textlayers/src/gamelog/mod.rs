use rltk::prelude::*;
mod builder;
pub use builder::*;
mod logstore;
use logstore::*;
pub use logstore::{clear_log, clone_log, restore_log, print_log};
use serde::{Serialize, Deserialize};
mod events;
pub use events::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color : RGB,
    pub text : String
}
