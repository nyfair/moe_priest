use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub use bevy_spine42 as bevy_spine;

const ADVFONT: &str = "TT_NPTelopMin-E.ttf";
const AMBIENCE: &str = "advscene/resources/advscene/sound/ambience/";
const ADVUI: &str = "AdvScene.png";
const MAXCHARA: u128 = 99;
const FBCHARA: u8 = 28;

// HSe StopHSe BgVoice StopBgVoice

fn get_intro() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut seed = nanos;
    let s = (nanos % 2) + 1;
    for _ in 0..10 {
        // SplitMix64
        seed = seed.wrapping_add(0x9E3779B97F4A7C15);
        let mut x = seed;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
        let random_val = x ^ (x >> 31);
        let num = (random_val % MAXCHARA) + 1;
        let p = format!("character/ch_1{:04}/general/vo_general_1{:04}_06{}.m4a", num, num, s);
        if Path::new(&format!("assets/advscene/resources/advscene/sound/voice/{}", p)).exists() {
            return p.to_string();
        }
    }
    format!("character/ch_1{:04}/general/vo_general_1{:04}_06{}.m4a", FBCHARA, FBCHARA, s).to_string()
}

include!("game.rs");
