pub use bevy_spine38 as bevy_spine;

const ADVFONT: &str = "FOT-NewRodinProN-EB.otf";
const AMBIENCE: &str = "advscene/resources/advscene/sound/se/";
const ADVUI: &str = "AdvScene.png";

pub fn get_intro() -> String {
    "ch_30005/general/basic/30005_030.m4a".to_string()
}

include!("game.rs");
