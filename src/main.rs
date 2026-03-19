#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use std::path::Path;

mod tween;
mod utage4;
mod monmusu;
mod rlyeh;

fn main() {
    let p = format!("assets/advscene/resources/advscene/sound/voice/{}", monmusu::get_intro());
    if Path::new(&p).exists() {
        monmusu::play();
    } else {
        rlyeh::play();
    }
}
