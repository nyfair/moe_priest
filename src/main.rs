#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;

mod tween;
mod utage4;
mod monmusu;
mod rlyeh;

fn main() {
    let path = Path::new("assets/advscene/resources/advscene/sound/voice/ch_30005/general/basic/30005_030.m4a");
    if path.exists() {
        monmusu::play();
    } else {
        rlyeh::play();
    }
}
