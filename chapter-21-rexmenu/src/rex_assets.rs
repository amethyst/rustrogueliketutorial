use rltk::{rex::XpFile};
use std::fs::File;

pub struct RexAssets {
    pub menu : XpFile
}

impl RexAssets {
    pub fn new() -> RexAssets {
        RexAssets{
            menu : XpFile::read(&mut File::open("../resources/SmallDungeon_80x50.xp").unwrap()).unwrap()
        }
    }
}