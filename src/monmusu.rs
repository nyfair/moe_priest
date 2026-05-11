pub use bevy_spine38 as bevy_spine;

const ADVFONT: &str = "FOT-NewRodinProN-EB.otf";
const AMBIENCE: &str = "advscene/resources/advscene/sound/se/";
const CHARTEXT: Color = Color::srgb_u8(237, 221, 192);
const VNTEXT: Color = Color::srgb_u8(78, 72, 70);

pub fn get_intro() -> String {
    "ch_30005/general/basic/30005_030.m4a".to_string()
}

pub fn get_adv_ui(asset_server: &Res<AssetServer>) -> Sprite {
    Sprite {
        image: asset_server.load("AdvScene.png"),
        color: Color::srgba(1., 1., 1., 0.6),
        ..default()
    }
}

pub fn get_adv_transform() -> Transform {
    Transform::from_translation(Vec3::new(0., -457., Z_UI as f32)).with_scale(Vec3::ONE)
}

include!("game.rs");
