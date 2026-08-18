use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub use bevy_spine42 as bevy_spine;

const ADVFONT: &str = "TT_NPTelopMin-E.ttf";
const AMBIENCE: &str = "advscene/resources/advscene/sound/ambience/";
const CHARTEXT: Color = Color::srgb_u8(200, 200, 200);
const VNTEXT: Color = CHARTEXT;
const SELECTBG: Color = Color::srgb_u8(24, 24, 24);
const SELECTBORDER: Color = Color::srgb_u8(0, 0, 0);

const MAXCHARA: u128 = 99;
const FBCHARA: u8 = 28;

pub fn get_intro() -> String {
    let micros = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let mut seed = micros;
    let s = (micros % 2) + 1;
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

pub fn get_adv_ui(asset_server: &Res<AssetServer>) -> Sprite {
    Sprite {
        image: asset_server.load("adv_base.png"),
        color: Color::srgba(1., 1., 1., 1.),
        ..default()
    }
}

pub fn get_adv_transform() -> Transform {
    Transform::from_translation(Vec3::new(0., -442., Z_UI as f32)).with_scale(Vec3::ONE)
}

include!("game.rs");
