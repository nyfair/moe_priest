// use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
// use bevy::window::PresentMode;

use bevy::audio::{PlaybackMode, Volume};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar, ScrollbarPlugin};
use bevy::window::{PrimaryWindow, WindowMode, WindowResolution};
use bevy_auto_scaling::{AspectRatio, ScalePlugin, ScalingUI, fixed_size_2d};
use bevy_spine::prelude::*;
use bevy_transform_interpolation::prelude::*;
use bevy_tweening::{AnimTarget, Lens, TweenAnim, TweeningPlugin, lens::*};
use regex::{Regex, Captures};
use std::collections::{BTreeMap, HashMap};
use std::fs::read_to_string;
use std::time::Duration;

use crate::utage4::{self, VNConfig};
use crate::tween::{Tween, TweenType};

include!("spine_tween.rs");

const FONT: &str = "FOT-NewRodinProN-EB.otf";
const HEADTEXT: Color = Color::srgb(0.5, 0.8, 0.7);
const LISTTEXT: Color = Color::srgb(0.2, 0.8, 0.2);
const SELECTTEXT: Color = Color::srgb(0.8, 0.8, 0.8);
const HOVERBG: Color = Color::srgb(0.1, 0.4, 0.1);
const VNSPEED: Duration = Duration::from_millis(60);
const AUTOFORWARD: Duration = Duration::from_millis(1000);
const Z_CG: i32 = 300;
const Z_UI: i32 = 993;
const Z_TEXT: i32 = 996;
const Z_FADE: i32 = 999;
const BG_SCALE: f32 = 1.725;
const EVENT_SCALE: f32 = 1.35;
const SPRITE_SCALE: f32 = 1.;
const SPINE_SCALE: f32 = 1.5;

macro_rules! str {
    ($var:expr) => { $var.as_deref().unwrap_or("") };
    ($var:expr, $default:expr) => { $var.as_deref().unwrap_or($default) };
}

macro_rules! f32 {
    ($var:ident = $source:expr, $default:expr) => {
        let $var = str!($source, stringify!($default)).parse::<f32>().unwrap_or($default);
    };
    ($var:ident, $default:expr) => {
        $var.parse::<f32>().unwrap_or($default)
    };
}

macro_rules! define_paths {
    ($root:literal, $(($name:ident, $subpath:literal)),*) => {
        $(
            const $name: &str = concat!($root, $subpath);
        )*
    };
}

define_paths! {
    "advscene/resources/advscene/sound/",
    (BGM, "bgm/"),
    (SE, "se/"),
    (VOICE, "voice/")
}

define_paths! {
    "advscene/resources/advscene/texture/",
    (BG, "bg/"),
    (EVENT, "event/"),
    (SPRITE, "sprite/")
}

#[derive(Debug)]
struct Location {
    path: String,
    name: String,
    ext: String,
}

#[derive(Clone, PartialEq)]
enum ListMode {
    Gallery,
    Motion,
    Memory,
}

#[derive(Resource)]
struct ViewRes {
    spines: BTreeMap<String, Location>,
    events: BTreeMap<String, Location>,
    mode: ListMode,
    vn: VNConfig,
    avg: bool,
    avg_nodes: Vec<utage4::Node>,
    avg_offset: usize,
    avg_regex: Regex,
    fast: bool,
    auto: bool,
    voice_played: bool,
    forwarded: bool,
    spine_cache: Vec<Entity>,
    wait_timer: Option<Timer>,
    params: HashMap<String, String>
}

#[derive(Component)]
struct SceneMenuList;

#[derive(Component)]
struct SceneMenu;

#[derive(Component)]
struct AnimeMenuList;

#[derive(Component)]
struct AnimeMenu;

#[derive(Component)]
struct ModeMenu;

#[derive(Component)]
struct VNChar;

#[derive(Component)]
struct VNText {
    text: String,
    index: usize,
    timer: Timer,
}

impl VNText {
    fn new() -> Self {
        Self {
            text: String::new(),
            index: 0,
            timer: Timer::new(VNSPEED, TimerMode::Once),
        }
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    fn update(&mut self, text: &str) {
        self.text = text.into();
        self.index = 0;
        self.timer = Timer::new(VNSPEED, TimerMode::Repeating);
    }

    fn skip_to_end(&mut self) {
        let l = self.len();
        if l > 1 {
            self.index = l - 1;
        }
    }

    fn finished(&self) -> bool {
        self.index >= self.len()
    }
}

#[derive(Component, Debug)]
struct FadeOverlay {
    color: Color,
    timer: Timer,
    fade_out: bool,
}

impl FadeOverlay {
    fn new(arg1: &str, arg6: &str, fade_out: bool) -> Self {
        let color: Color = Srgba::hex(arg1).unwrap_or(Srgba::WHITE).into();
        let timer = Timer::from_seconds(f32!(arg6, 0.2), TimerMode::Once);
        Self {
            color,
            timer,
            fade_out,
        }
    }

    fn init_color(&mut self) -> Color {
        if self.fade_out {
            self.color.set_alpha(1.);
        }
        self.color
    }
}

#[derive(PartialEq)]
enum TextureType {
    Bg,
    Event,
    Sprite,
}

#[derive(Component)]
struct AvgTransform {
    orig: Transform,
    avg: Transform,
}

#[derive(Component)]
// category, label, layer, scale, avg_info
struct VNTexture(TextureType, String, String, f32, AvgTransform);

impl VNTexture {
    fn move_to(&mut self, to: Vec3, update: bool) -> Vec3 {
        if update {
            self.4.avg.translation = to;
        }
        vec3(
            self.4.orig.translation.x + self.3 * to.x,
            self.4.orig.translation.y + self.3 * to.y,
            to.z,
        )
    }

    fn move_by(&mut self, by: Vec3) -> Vec3 {
        self.4.avg.translation += by;
        vec3(
            self.4.orig.translation.x + self.3 * self.4.avg.translation.x,
            self.4.orig.translation.y + self.3 * self.4.avg.translation.y,
            self.4.orig.translation.z + by.z,
        )
    }

    fn scale_to(&mut self, to: Vec3, update: bool) -> Vec3 {
        if update {
            self.4.avg.scale = to;
        }
        self.4.orig.scale * to
    }

    fn scale_by(&mut self, by: Vec3) -> Vec3 {
        self.4.avg.scale *= by;
        self.4.orig.scale * self.4.avg.scale
    }
}

#[derive(Component)]
// label, animation, layer, avg_info
struct VNSpine(String, String, String, AvgTransform);

impl VNSpine {
    fn move_to(&mut self, to: Vec3, update: bool) -> Vec3 {
        if update {
            self.3.avg.translation = to;
        }
        vec3(
            self.3.orig.translation.x + SPINE_SCALE * to.x,
            self.3.orig.translation.y + SPINE_SCALE * to.y * 0.5,
            to.z,
        )
    }

    fn move_by(&mut self, by: Vec3) -> Vec3 {
        self.3.avg.translation += by;
        vec3(
            self.3.orig.translation.x + SPINE_SCALE * self.3.avg.translation.x,
            self.3.orig.translation.y + SPINE_SCALE * self.3.avg.translation.y * 0.5,
            self.3.orig.translation.z + by.z,
        )
    }

    fn scale_to(&mut self, to: Vec3, update: bool) -> Vec3 {
        if update {
            self.3.avg.scale = to;
        }
        self.3.orig.scale * to
    }

    fn scale_by(&mut self, by: Vec3) -> Vec3 {
        self.3.avg.scale *= by;
        self.3.orig.scale * self.3.avg.scale
    }
}

#[derive(PartialEq)]
enum AudioType {
    Bgm,
    Se,
    Ambience,
    Voice,
}

#[derive(Component)]
// category, label
struct VNAudio(AudioType, String);

#[derive(Component)]
struct AudioFade(Timer, Volume);

#[derive(Component)]
struct VNGui;

#[derive(Message)]
struct SceneMsg(ListMode);

#[derive(Message)]
struct VNToogleMsg(bool);

#[derive(Message)]
struct VNMsg;

pub fn play() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    // present_mode: PresentMode::Immediate,
                    resolution: WindowResolution::new(3840, 2160).with_scale_factor_override(1.),
                    ..default()
                }),
                ..default()
            }
        ),
            // FrameTimeDiagnosticsPlugin::default(),
            // LogDiagnosticsPlugin::default(),
            ScalePlugin,
            ScrollbarPlugin,
            SpinePlugin,
            TransformInterpolationPlugin::interpolate_all(),
            TweeningPlugin,
        ))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Time::<Fixed>::from_hz(30.))
        .insert_resource(ScalingUI {
            width: 3840.,
            height: 2160.,
        })
        .add_message::<SceneMsg>()
        .add_message::<VNToogleMsg>()
        .add_message::<VNMsg>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            toggle_fullscreeen,
            list_scene,
            choose_scene,
            spine_spawn.in_set(SpineSet::OnReady),
            choose_animation,
            choose_mode,
            input_handler,
            toggle_vn,
            vn_dialogue,
            fade_overlay,
            fade_sound,
            check_wait,
            check_auto_forward,
        ))
        .add_systems(FixedUpdate, (mouse_scroll, mouse_object_move, play_vn))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut scene_msg: MessageWriter<SceneMsg>,
) {
    let vn = if let Ok(content) = read_to_string("assets/advscene/scenariochapter/config.chapter.json") {
        VNConfig::new(content)
    } else {
        VNConfig::default()
    };

    let mut spines = BTreeMap::new();
    if let Ok(content) = read_to_string("assets/spine.txt") {
        for spine in content.lines() {
            if let (Some(l), Some(r)) = (spine.rfind('/'), spine.rfind('.'))
                && l < r {
                    let path = spine[..l].to_string();
                    if let Some(rr) = path.rfind('/') {
                        let key = path[rr+1..].to_string();
                        let name = spine[l+1..r].to_string();
                        let ext = spine[r+1..].to_string();
                        spines.insert(key, Location {
                            path,
                            name,
                            ext,
                        });
                    }
                }
        }
    }
    let mut events = BTreeMap::new();
    if let Ok(content) = read_to_string("assets/memory.txt") {
        for event in content.lines() {
            if let (Some(l), Some(r)) = (event.rfind('/'), event.find('.')) && l < r {
                let path = event[..l].to_string();
                let name = event[l+1..r].to_string();
                let ext = event[r+1..].to_string();
                events.insert(name.clone(), Location {
                    path,
                    name,
                    ext,
                });
            }
        }
    }

    commands.spawn((
        Camera2d,
        AspectRatio(16. / 9.),
        fixed_size_2d(1920. * 1.14514, 1080. * 1.14514),
    ));
    commands.insert_resource(ViewRes {
        spines,
        events,
        mode: ListMode::Gallery,
        vn,
        avg: false,
        avg_nodes: vec!(),
        avg_offset: 0,
        // <interval=???> to ..., <param=???> for param matching, remove other tags
        avg_regex: Regex::new(r"<interval=(?P<interval>[^>]*)>|<param=(?P<param>[^>]*)>|(?P<other><[^>]*>)").unwrap(),
        fast: false,
        auto: false,
        voice_played: false,
        forwarded: false,
        spine_cache: vec!(),
        wait_timer: None,
        params: HashMap::new(),
    });

    commands.spawn((
        Visibility::Visible,
        ZIndex(Z_UI),
        Node {
            width: Val::Percent(11.),
            height: Val::Percent(26.),
            left: Val::Percent(1.),
            bottom: Val::Percent(1.),
            flex_direction: FlexDirection::ColumnReverse,
            align_self: AlignSelf::End,
            row_gap: Val::Percent(1.),
            ..default()
        },
    )).with_children(|parent| {
        for m in ["Gallery", "Motion", "Memory"] {
            parent.spawn((
                Button,
                Text::new(m),
                ModeMenu,
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 42.,
                    ..default()
                },
                TextColor(HEADTEXT),
                BackgroundColor(Color::NONE),
            ));
        }
    });
    scene_msg.write(SceneMsg(ListMode::Gallery));
    commands.spawn((
        AudioPlayer::new(
            asset_server.load(format!("{}{}", VOICE, get_intro()))
        ),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));

    commands.spawn((
        Visibility::Hidden,
        VNGui,
        get_adv_ui(&asset_server),
        get_adv_transform(),
    ));
    commands.spawn((
        Visibility::Hidden,
        ZIndex(Z_TEXT),
        Text2d::new(""),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::CENTER_LEFT,
        VNGui,
        VNChar,
        TextFont {
            font: asset_server.load(ADVFONT),
            font_size: 136.,
            ..default()
        },
        TextColor(CHARTEXT),
        Transform::from_translation(Vec3::new(-572., -366., Z_UI as f32)).with_scale(Vec3::ONE / 4.),
    ));
    commands.spawn((
        Visibility::Hidden,
        ZIndex(Z_TEXT),
        Text2d::new(""),
        TextLayout::new_with_justify(Justify::Left),
        Anchor::TOP_LEFT,
        VNGui,
        VNText::new(),
        TextFont {
            font: asset_server.load(ADVFONT),
            font_size: 136.,
            ..default()
        },
        TextColor(VNTEXT),
        Transform::from_translation(Vec3::new(-550., -420., Z_UI as f32)).with_scale(Vec3::ONE / 4.),
    ));
}

fn toggle_fullscreeen(
    key: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if (key.just_released(KeyCode::Enter) && (key.pressed(KeyCode::AltLeft) || key.pressed(KeyCode::AltRight)))
    || key.just_released(KeyCode::F11) {
        if window.mode == WindowMode::Windowed {
            window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
        } else {
            window.mode = WindowMode::Windowed;
        }
    }
}

fn list_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    scene_query: Query<Entity, With<SceneMenuList>>,
    mut scene_msg: MessageReader<SceneMsg>,
    view_res: Res<ViewRes>,
) {
    if let Some(event) = scene_msg.read().last() {
        scene_query.iter().for_each(|entity| {
            commands.entity(entity).despawn()
        });
        commands.spawn((
            Visibility::Visible,
            SceneMenuList,
            ZIndex(Z_UI),
            Node {
                width: Val::Percent(11.),
                height: Val::Percent(96.),
                left: Val::Percent(88.),
                top: Val::Percent(1.),
                align_items: AlignItems::End,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Select Scenario"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 42.,
                    ..default()
                },
                TextColor(HEADTEXT),
                TextLayout::new_with_justify(Justify::Right),
            ));
            parent.spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
                grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
                ..default()
            }).with_children(|parent| {
                let scrollable = parent.spawn((
                    Node {
                        align_items: AlignItems::End,
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                )).with_children(|parent| {
                    let (items, scene_filter):
                    (&BTreeMap<_, _>, fn(&&String) -> bool) = match event.0 {
                        ListMode::Memory => (&view_res.events, |_| true),
                        ListMode::Gallery => (&view_res.spines, |x| x.starts_with("r18")),
                        ListMode::Motion => (&view_res.spines, |x| !x.starts_with("r18")),
                    };
                    for bundle_name in items.keys().filter(scene_filter) {
                        parent.spawn((
                            Button,
                            Text::new(bundle_name),
                            SceneMenu,
                            TextFont {
                                font: asset_server.load(FONT),
                                font_size: 35.,
                                ..default()
                            },
                            TextColor(LISTTEXT),
                            BackgroundColor(Color::NONE),
                            TextLayout::new_with_justify(Justify::Right),
                        ));
                    }
                }).id();
                parent.spawn((
                    Node {
                        min_width: px(12),
                        ..default()
                    },
                    Scrollbar {
                        orientation: ControlOrientation::Vertical,
                        target: scrollable,
                        min_thumb_length: 48.,
                    },
                    Children::spawn(Spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(HOVERBG),
                        CoreScrollbarThumb,
                    ))),
                ));
            });
        });
    }
}

fn choose_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut TextColor,
        &mut BackgroundColor,
        &SceneMenu,
    ), (Changed<Interaction>, With<Button>),>,
    spine_query: Query<Entity, With<Spine>>,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut vn_ui_msg: MessageWriter<VNToogleMsg>,
    mut view_res: ResMut<ViewRes>,
) {
    interaction_query.iter_mut().for_each(|(interaction, text, mut color, mut bg_color, _)| {
        match *interaction {
            Interaction::Pressed => {
                let bundle_name = &text.to_string();
                if view_res.mode == ListMode::Memory {
                    if let Some(file) = view_res.events.get(bundle_name) && let Ok(content) =
                            read_to_string(format!("assets/{}/{}.{}", file.path, file.name, file.ext)) {
                        let book = utage4::parse_book(content);
                        view_res.avg = true;
                        view_res.avg_nodes = book;
                        view_res.avg_offset = 0;
                        view_res.fast = false;
                        view_res.params = HashMap::new();
                        vn_ui_msg.write(VNToogleMsg(true));
                    }
                } else if let Some(file) = view_res.spines.get(bundle_name) {
                    let skeleton = if file.ext == "skel" {
                        SkeletonData::new_from_binary(
                            asset_server.load(format!("{}/{}.{}", file.path, file.name, file.ext)),
                            asset_server.load(format!("{}/{}.atlas", file.path, file.name)),
                        )
                    } else {
                        SkeletonData::new_from_json(
                            asset_server.load(format!("{}/{}.{}", file.path, file.name, file.ext)),
                            asset_server.load(format!("{}/{}.atlas", file.path, file.name)),
                        )
                    };
                    let skeleton_handle = skeletons.add(skeleton);
                    spine_query.iter().for_each(|entity| {
                        commands.entity(entity).despawn()
                    });
                    commands.spawn((
                        SkeletonDataHandle(skeleton_handle.clone()),
                        Transform::from_xyz(0., 0., Z_CG as f32).with_scale(Vec3::ONE * 0.5),
                    ));
                }
            }
            Interaction::Hovered => {
                *color = SELECTTEXT.into();
                *bg_color = HOVERBG.into();
            }
            _ => {
                *color = LISTTEXT.into();
                *bg_color = Color::NONE.into();
            }
        }
    });
}

fn spine_spawn(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut spine_query: Query<&mut Spine, Without<VNSpine>>,
    mut vn_spine_query: Query<(&mut Spine, &VNSpine)>,
    anime_query: Query<Entity, With<AnimeMenuList>>,
    mut spine_visibility: Query<&mut Visibility, With<Spine>>,
    mut spine_ready_msg: MessageReader<SpineReadyMsg>,
    view_res: Res<ViewRes>,
) {
    if view_res.mode != ListMode::Memory {
        for msg in spine_ready_msg.read() {
            anime_query.iter().for_each(|entity| {
                commands.entity(entity).despawn()
            });
            let mut animation_list = vec!();
            if let Ok(mut spine) = spine_query.get_mut(msg.entity) {
                let Spine(SkeletonController {
                    animation_state,
                    ..
                }) = spine.as_mut();
                for i in animation_state.data().skeleton_data().animations() {
                    animation_list.push(i.name().to_string());
                }
            }

            commands.spawn((
                Visibility::Visible,
                AnimeMenuList,
                ZIndex(Z_UI),
                Node {
                    width: Val::Percent(11.),
                    height: Val::Percent(66.),
                    left: Val::Percent(1.),
                    top: Val::Percent(1.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    Button,
                    Text::new("Select Action"),
                    TextFont {
                        font: asset_server.load(FONT),
                        font_size: 42.,
                        ..default()
                    },
                    TextColor(HEADTEXT),
                    BackgroundColor(Color::NONE),
                ));
                for animation in animation_list {
                    parent.spawn((
                        Button,
                        Text::new(animation),
                        AnimeMenu,
                        TextFont {
                            font: asset_server.load(FONT),
                            font_size: 35.,
                            ..default()
                        },
                        TextColor(LISTTEXT),
                        BackgroundColor(Color::NONE),
                    ));
                }
            });
        }
    } else if view_res.avg {
        for msg in spine_ready_msg.read() {
            if let Ok((mut spine, s)) = vn_spine_query.get_mut(msg.entity)
                && let Ok(mut visibility) = spine_visibility.get_mut(msg.entity) {
                if &s.1 == "<Off>" {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Visible;
                    let _ = spine.animation_state.set_animation_by_name(0, &s.1, true);
                }
            }
        }
    }
}

fn choose_animation(
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut TextColor,
        &mut BackgroundColor,
        &AnimeMenu,
    ), (Changed<Interaction>, With<Button>),>,
    mut spine_query: Query<&mut Spine, Without<VNSpine>>,
) {
    interaction_query.iter_mut().for_each(|(interaction, text, mut color, mut bg_color, _)| {
        match *interaction {
            Interaction::Pressed => {
                spine_query.iter_mut().for_each(|mut spine| {
                    let _ = spine.animation_state.set_animation_by_name(0, text, true);
                })
            }
            Interaction::Hovered => {
                *color = SELECTTEXT.into();
                *bg_color = HOVERBG.into();
            }
            _ => {
                *color = LISTTEXT.into();
                *bg_color = Color::NONE.into();
            }
        }
    })
}

fn choose_mode(
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut BackgroundColor,
        &ModeMenu,
    ), (Changed<Interaction>, With<Button>),>,
    mut scene_msg: MessageWriter<SceneMsg>,
    mut view_res: ResMut<ViewRes>,
) {
    interaction_query.iter_mut().for_each(|(interaction, text, mut bg_color, _)| {
        match *interaction {
            Interaction::Pressed => {
                let mode = match text.as_str() {
                    "Motion" => ListMode::Motion,
                    "Memory" => ListMode::Memory,
                    _ => ListMode::Gallery,
                };
                view_res.mode = mode.clone();
                scene_msg.write(SceneMsg(mode));
            }
            Interaction::Hovered => {
                *bg_color = HOVERBG.into();
            }
            _ => {
                *bg_color = Color::NONE.into();
            }
        }
    })
}

fn input_handler(
    mut viewer_ui: Query<&mut Visibility, Without<VNGui>>,
    mut vn_ui: Query<&mut Visibility, With<VNGui>>,
    mut vn_ui_msg: MessageWriter<VNToogleMsg>,
    mut vn_msg: MessageWriter<VNMsg>,
    button: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut view_res: ResMut<ViewRes>,
) {
    if button.just_pressed(MouseButton::Right) {
        if view_res.avg {
            vn_ui.iter_mut().for_each(|mut v| {
                v.toggle_visible_hidden()
            })
        } else {
            viewer_ui.iter_mut().for_each(|mut v| {
                v.toggle_visible_hidden()
            })
        }
    }

    if view_res.avg {
        if button.just_pressed(MouseButton::Left)
        || key.just_pressed(KeyCode::Enter) || key.just_pressed(KeyCode::Space) {
            vn_msg.write(VNMsg);
        }
        if key.just_pressed(KeyCode::Escape) {
            view_res.avg = false;
            view_res.wait_timer = None;
            vn_ui_msg.write(VNToogleMsg(false));
        }
        if key.just_released(KeyCode::ControlLeft) || key.just_released(KeyCode::ControlRight) {
            view_res.fast = false;
        }
        if key.pressed(KeyCode::ControlLeft) || key.pressed(KeyCode::ControlRight) {
            view_res.fast = true;
        }
        if key.just_released(KeyCode::Tab) {
            view_res.auto = !view_res.auto;
        }
    }
}

fn toggle_vn(
    mut commands: Commands,
    mut viewer_ui: Query<&mut Visibility, Without<VNGui>>,
    mut vn_ui: Query<&mut Visibility, With<VNGui>>,
    mut text: Single<&mut Text2d, With<VNText>>,
    mut vn_text: Single<&mut VNText>,
    despawn_query: Query<Entity, Or<(With<Spine>, With<AnimeMenuList>)>>,
    vn_despawn_query: Query<Entity, Or<(With<FadeOverlay>, With<VNTexture>, (With<VNAudio>, Without<AudioFade>))>>,
    mut vn_ui_msg: MessageReader<VNToogleMsg>,
    mut vn_msg: MessageWriter<VNMsg>,
) {
    if let Some(msg) = vn_ui_msg.read().last() {
        despawn_query.iter().for_each(|entity| {
            commands.entity(entity).despawn()
        });
        if msg.0 {
            for mut v in &mut viewer_ui {
                *v = Visibility::Hidden
            }
            vn_msg.write(VNMsg);
        } else {
            text.0 = String::new();
            vn_text.text = String::new();
            vn_despawn_query.iter().for_each(|entity| {
                commands.entity(entity).despawn()
            });
            vn_ui.iter_mut().for_each(|mut v| {
                *v = Visibility::Hidden
            });
            viewer_ui.iter_mut().for_each(|mut v| {
                *v = Visibility::Visible
            });
        }
    }
}

fn vn_dialogue(
    mut vn_text: Single<(&mut Text2d, &mut VNText)>,
    fade_query: Query<&FadeOverlay>,
    time: Res<Time>,
    view_res: Res<ViewRes>,
) {
    if view_res.avg && fade_query.count() == 0 {
        vn_text.1.timer.tick(time.delta());
        if vn_text.1.timer.just_finished() && vn_text.1.index < vn_text.1.len() {
            vn_text.1.index += 1;
            let displayed_text: String = vn_text.1.text
                .chars()
                .take(vn_text.1.index)
                .collect();
            vn_text.0.0 = displayed_text.clone();
        }
    }
}

fn fade_overlay(
    mut commands: Commands,
    mut fade_query: Query<(Entity, &mut FadeOverlay, &mut BackgroundColor)>,
    time: Res<Time>,
) {
    fade_query.iter_mut().for_each(|(entity, mut fade, mut color)| {
        fade.timer.tick(time.delta());
        if fade.timer.just_finished() {
            info!("layer fade{} effect finished", if fade.fade_out {"out"} else {"in"});
            commands.entity(entity).despawn();
        } else {
            let mut new_color = fade.color;
            if fade.fade_out {
                new_color.set_alpha(fade.timer.fraction());
            } else {
                new_color.set_alpha(fade.timer.fraction_remaining());
            }
            *color = BackgroundColor(new_color);
        }
    })
}

fn fade_sound(
    mut commands: Commands,
    mut audio_query: Query<(Entity, &mut AudioSink, &mut AudioFade)>,
    time: Res<Time>,
) {
    for (entity, mut sink, mut fade) in audio_query.iter_mut() {
        fade.0.tick(time.delta());
        if fade.0.is_finished() {
            info!("sound fade out");
            commands.entity(entity).despawn();
        } else {
            sink.set_volume(
                fade.1.fade_towards(Volume::Linear(0.), fade.0.fraction()),
            );
        }
    }
}

fn check_wait(
    mut vn_msg: MessageWriter<VNMsg>,
    spine_query: Query<(), With<Spine>>,
    time: Res<Time>,
    mut view_res: ResMut<ViewRes>,
) {
    if view_res.avg {
        if !view_res.spine_cache.is_empty() {
            if view_res.spine_cache.iter().all(|&s| spine_query.contains(s)) {
                view_res.spine_cache = vec!();
            } else {
                return;
            }
        }
        if view_res.fast {
            view_res.wait_timer = None;
            vn_msg.write(VNMsg);
        } else if let Some(timer) = &mut view_res.wait_timer {
            timer.tick(time.delta());
            if timer.is_finished() {
                view_res.wait_timer = None;
                vn_msg.write(VNMsg);
            }
        }
    }
}

fn check_auto_forward(
    vn_text: Single<&VNText>,
    audio_query: Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
    mut view_res: ResMut<ViewRes>,
) {
    if view_res.avg && view_res.auto && !view_res.fast && view_res.wait_timer.is_none()
        && vn_text.finished() && !view_res.forwarded {
            for (_, _, audio) in audio_query.iter() {
                if audio.0 == AudioType::Voice {
                    view_res.voice_played = true;
                    return
                }
            }
            if view_res.voice_played {
                view_res.wait_timer = Some(Timer::from_seconds(0., TimerMode::Once));
                view_res.voice_played = false;
            } else {
                view_res.wait_timer = Some(Timer::new(AUTOFORWARD, TimerMode::Once));
            }
            view_res.forwarded = true;
        }
}

fn mouse_scroll(
    mut spine_query: Query<&mut Transform, Or<(With<Spine>, With<VNTexture>)>>,
    scrollbar: Single<&Scrollbar>,
    mut scrolled_query: Query<(&mut ScrollPosition, &ComputedNode), Without<Scrollbar>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut scroll: MessageReader<MouseWheel>,
    time: Res<Time>,
) {
    for ev in scroll.read() {
        if ev.y == 0. {
            break
        }
        let delta_secs = time.delta_secs();
        if let Some(pos) = window.cursor_position() {
            if pos.x > window.width() * 0.88 {
                if let Ok((mut scroll_pos, scroll_content)) = scrolled_query.get_mut(scrollbar.target) {
                    let visible_size = scroll_content.size() * scroll_content.inverse_scale_factor;
                    let content_size = scroll_content.content_size() * scroll_content.inverse_scale_factor;
                    let range = (content_size.y - visible_size.y).max(0.);
                    scroll_pos.y -= ev.y * 5000. * delta_secs;
                    scroll_pos.y = scroll_pos.y.clamp(0., range);
                };
            } else {
                spine_query.iter_mut().for_each(|mut spine| {
                    spine.scale += ev.y * 0.1 * delta_secs
                });
            }
        }
    }
}

fn mouse_object_move(
    mut object_query: Query<&mut Transform, Or<(With<Spine>, With<VNTexture>)>>,
    mut motion: MessageReader<MouseMotion>,
    button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if button.pressed(MouseButton::Middle) {
        let delta_secs = time.delta_secs();
        for ev in motion.read() {
            object_query.iter_mut().for_each(|mut obj| {
                obj.translation.x += ev.delta.x * 6. * delta_secs;
                obj.translation.y -= ev.delta.y * 6. * delta_secs;
            })
        }
    }
}

fn play_vn(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut vn_char: Single<&mut Text2d, With<VNChar>>,
    mut vn_text: Single<(&mut Text2d, &mut VNText), Without<VNChar>>,
    mut vn_ui: Query<&mut Visibility, With<VNGui>>,
    mut audio_query: Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
    mut tex_query: Query<(Entity, &mut VNTexture, &mut Transform, &mut Sprite), (Without<VNSpine>, Without<VNGui>)>,
    mut spine_query: Query<(Entity, &mut Spine, &mut VNSpine, &mut Transform), (Without<VNTexture>, Without<VNGui>)>,
    mut gui_query: Query<(Entity, &VNGui, &mut Transform), (Without<VNSpine>, Without<VNTexture>)>,
    mut spine_visibility: Query<&mut Visibility, (With<Spine>, Without<VNGui>)>,
    mut vn_msg: MessageReader<VNMsg>,
    mut vn_ui_msg: MessageWriter<VNToogleMsg>,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut view_res: ResMut<ViewRes>,
) {
    if vn_msg.read().last().is_some() {
        if !view_res.fast && (view_res.wait_timer.is_some() || !view_res.spine_cache.is_empty()) {
            return
        }
        if vn_text.1.finished() {
            while view_res.avg_offset < view_res.avg_nodes.len() {
                let node = &view_res.avg_nodes[view_res.avg_offset];
                info!("{:?}", node);
                match node.command.as_ref().map(|s| &s[..]) {
                    None => {
                        let (wait, entity) = default_cmd(
                            node, &asset_server, &mut commands, &mut vn_char, &mut vn_text, &mut vn_ui,
                            &mut audio_query, &mut spine_query, &mut spine_visibility, &mut skeletons, &view_res);
                        if let Some(entity) = entity {
                            view_res.spine_cache.push(entity);
                        }
                        if wait {
                            view_res.avg_offset += 1;
                            view_res.forwarded = false;
                            break;
                        }
                    }
                    Some("CharacterOff") => {
                        character_off_cmd(node, &mut commands, &mut spine_query, true);
                    }
                    Some(f @ "Bg") | Some(f @ "BgEvent") | Some(f @ "Sprite") => {
                        img_cmd(f, node, &asset_server, &mut commands, &view_res);
                    }
                    Some(f @ "BgOff") | Some(f @ "BgEventOff") => {
                        bg_off_cmd(f, &mut commands, &mut tex_query);
                    }
                    Some("SpriteOff") => {
                        sprite_off_cmd(node, &mut commands, &mut tex_query);
                    }
                    Some("LayerOff") => {
                        layer_off_cmd(node, &mut commands, &mut tex_query, &mut spine_query);
                    }
                    Some(f @ "Se") | Some(f @ "Bgm") | Some(f @ "Ambience")
                    | Some(f @ "HSe") | Some(f @ "BgVoice") => {
                        sound_cmd(f, node, &asset_server, &mut commands, &mut audio_query, &view_res);
                    }
                    Some(f @ "StopSe") | Some(f @ "StopBgm") | Some(f @ "StopAmbience")
                    | Some(f @ "StopHSe") | Some(f @ "StopBgVoice") => {
                        stop_sound_item_cmd(f, node, &mut commands, &mut audio_query, false);
                    }
                    Some("Voice") => {
                        voice_cmd(node, &asset_server, &mut commands, &mut audio_query);
                    }
                    Some("StopVoice") => {
                        stop_voice_cmd(&mut commands, &mut audio_query);
                    }
                    Some("StopSound") => {
                        stop_sound_cmd(node, &mut commands, &mut audio_query);
                    }
                    Some("Wait") => {
                        f32!(t = node.arg6, 0.1);
                        view_res.wait_timer = Some(Timer::from_seconds(t, TimerMode::Once));
                        view_res.avg_offset += 1;
                        break;
                    }
                    Some(f @ "FadeOut") | Some(f @ "FadeIn") => {
                        fade_overlay_cmd(f, node, &mut commands);
                    }
                    Some("Param") => {
                        if let Some((k, v)) = param_cmd(node) {
                            view_res.params.insert(k, v);
                        }
                    }
                    Some("Tween") => {
                        if view_res.spine_cache.is_empty() {
                            tween_cmd(node, &mut commands, &mut spine_query, &mut tex_query, &mut gui_query);
                        } else {
                            // wait for spine spawn
                            view_res.wait_timer = Some(Timer::from_seconds(0., TimerMode::Once));
                            view_res.forwarded = true;
                            break;
                        }
                    }
                    Some(cmd) => warn!("Command {} Unimplemented", cmd)
                }
                view_res.avg_offset += 1;
            }
            if view_res.avg_offset >= view_res.avg_nodes.len() {
                view_res.avg = false;
                view_res.wait_timer = None;
                vn_ui_msg.write(VNToogleMsg(false));
            }
        } else {
            vn_text.1.skip_to_end();
            vn_ui.iter_mut().for_each(|mut v| {
                *v = Visibility::Visible
            })
        }
    }
}

fn normalize(text: &str, view_res: &ResMut<ViewRes>) -> String {
    view_res.avg_regex.replace_all(text, |caps: &Captures| {
        if let Some(p) = caps.name("interval") {
            let key = p.as_str();
            return key.parse::<f32>()
                .map(|num| (num / VNSPEED.as_secs_f32()).ceil())
                .map(|count| {
                    if count > 0.0 {
                        "\u{200c}".repeat(count as usize)
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();
        } else if let Some(p) = caps.name("param") {
            let key = p.as_str();
            return view_res.params.get(key)
                .map(ToString::to_string)
                .or_else(|| {
                    view_res.vn.param.get(key)
                        .and_then(|param| param.value.as_ref().map(ToString::to_string))
                })
                .unwrap_or_default();
        }
        String::new()
    }).into_owned()
}

fn default_cmd(
    node: &utage4::Node,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    vn_char: &mut Single<&mut Text2d, With<VNChar>>,
    vn_text: &mut Single<(&mut Text2d, &mut VNText), Without<VNChar>>,
    vn_ui: &mut Query<&mut Visibility, With<VNGui>>,
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
    spine_query: &mut Query<(Entity, &mut Spine, &mut VNSpine, &mut Transform), (Without<VNTexture>, Without<VNGui>)>,
    spine_visibility: &mut Query<&mut Visibility, (With<Spine>, Without<VNGui>)>,
    skeletons: &mut ResMut<Assets<SkeletonData>>,
    view_res: &ResMut<ViewRes>,
) -> (bool, Option<Entity>) {
    let mut wait = false;
    let mut spine_entity = None;
    // dialogue text
    if let Some(t) = &node.text {
        let text = normalize(t, view_res);
        vn_text.0.0 = "".into();
        vn_text.1.update(&text);
        vn_ui.iter_mut().for_each(|mut v| {
            *v = Visibility::Visible
        });
        wait = true;
    } else {
        vn_ui.iter_mut().for_each(|mut v| {
            *v = Visibility::Hidden
        });
    }
    // play voice
    if let Some(voice) = &node.voice {
        stop_voice_cmd(commands, audio_query);
        info!("play voice {}", voice);
        commands.spawn((
            VNAudio(AudioType::Voice, "".into()),
            AudioPlayer::new(
                asset_server.load(format!("{}{}.m4a", VOICE, voice))
            ),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(1.),
                ..default()
            },
        ));
    }
    // draw character and update dialogue character name
    let char_name = str!(node.arg1);
    if let (Some(character), Some(motion)) = (view_res.vn.character.get(char_name), node.arg2.as_deref()) {
        if let Some(name_text) = character.name_text.as_deref() {
            vn_char.0 = normalize(name_text, view_res)
        } else {
            vn_char.0 = char_name.into();
        }
        let mut spine_spawned = false;
        spine_query.iter_mut().for_each(|(e, mut spine, mut s, _)| {
            if s.0 == char_name {
                spine_spawned = true;
                s.1 = motion.into();
                if let Ok(mut visibility) = spine_visibility.get_mut(e) {
                    if s.1 == "<Off>" {
                        *visibility = Visibility::Hidden;
                    } else {
                        *visibility = Visibility::Visible;
                        let _ = spine.animation_state.set_animation_by_name(0, motion, true);
                    }
                }
            }
        });
        if !spine_spawned {
            info!("load chara {:?}", character);
            let layer = view_res.vn.layer.get(str!(node.arg3));
            // command arg + (layer > character > preset)
            f32!(x = layer.and_then(|l| l.x.as_deref()).or(character.x.as_deref()), 0.);
            f32!(y = layer.and_then(|l| l.y.as_deref()).or(character.y.as_deref()), 0.);
            f32!(z = layer.and_then(|l| l.order.as_deref()).or(character.z.as_deref()), 0.);
            f32!(scale_x = layer.and_then(|l| l.scale_x.as_deref()).or(character.scale.as_deref()), 1.);
            f32!(scale_y = layer.and_then(|l| l.scale_y.as_deref()).or(character.scale.as_deref()), 1.);
            let (off_x, off_y) = (str!(node.arg4), str!(node.arg5));
            let file_name = str!(character.file_name);
            if let (Some(l), Some(r)) = (file_name.rfind('/'), file_name.rfind('.')) && l < r {
                let path = file_name[..l].to_string();
                if let Some(rr) = path.rfind('/') {
                    let bundle_name = path[rr+1..].to_string().to_lowercase();
                    if let Some(file) = view_res.spines.get(&bundle_name) {
                        let skeleton = if file.ext == "skel" {
                            SkeletonData::new_from_binary(
                                asset_server.load(format!("{}/{}.{}", file.path, file.name, file.ext)),
                                asset_server.load(format!("{}/{}.atlas", file.path, file.name)),
                            )
                        } else {
                            SkeletonData::new_from_json(
                                asset_server.load(format!("{}/{}.{}", file.path, file.name, file.ext)),
                                asset_server.load(format!("{}/{}.atlas", file.path, file.name)),
                            )
                        };
                        let skeleton_handle = skeletons.add(skeleton);
                        spine_entity = Some(commands.spawn((
                            SkeletonDataHandle(skeleton_handle.clone()),
                            Transform::from_xyz((x + f32!(off_x, 0.)) * SPINE_SCALE,
                                                (y + f32!(off_y, 0.)) * SPINE_SCALE * 0.5, z)
                                .with_scale(Vec3::new(scale_x * SPINE_SCALE, scale_y * SPINE_SCALE, 1.)),
                            VNSpine(char_name.into(), motion.into(), str!(node.arg3).into(),
                                AvgTransform {
                                    orig: Transform::from_xyz(x * SPINE_SCALE, y * SPINE_SCALE * 0.5, z)
                                            .with_scale(Vec3::new(scale_x * SPINE_SCALE, scale_y * SPINE_SCALE, 1.)),
                                    avg: Transform::from_xyz(f32!(off_x, 0.), f32!(off_y, 0.), z).with_scale(Vec3::ONE)
                                }
                            )
                        )).id());
                    }
                }
            }
        }
    } else {
        vn_char.0 = char_name.into();
    }
    (wait, spine_entity)
}

fn fade_overlay_cmd(
    f: &str,
    node: &utage4::Node,
    commands: &mut Commands,
) {
    let mut overlay = FadeOverlay::new(
        str!(node.arg1, "#FFFFFF"),
        str!(node.arg6, "0.2"),
        matches!(f, "FadeOut")
    );
    let init_color = overlay.init_color();
    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(init_color),
        ZIndex(Z_FADE),
        overlay,
    ));
}

fn img_cmd(
    f: &str,
    node: &utage4::Node,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    view_res: &ResMut<ViewRes>,
) {
    /* Texture types in the reference sheet may differ from their usage in scripts.
    For example, a background (BG) might function as an sprite image in-game.
    Use the script's type for processing logic, but refer to the reference sheet for asset lookups. */
    // type for ecs query
    let (real_type, label_name) = match f {
        "Bg" => (TextureType::Bg, str!(node.arg1)),
        "BgEvent" => (TextureType::Event, str!(node.arg1)),
        "Sprite" => (TextureType::Sprite, str!(node.arg2)),
        _ => return
    };
    if let Some(texture) = view_res.vn.texture.get(label_name) {
        let layer = view_res.vn.layer.get(str!(node.arg3));
        // type for texture file search
        let texture_type = match texture.entry_type.as_deref() {
            Some("Bg") => "Bg",
            Some("Event") => "BgEvent",
            Some("Sprite") => "Sprite",
            _ => f,
        };
        let (img_path, scale) = match texture_type {
            "Bg" => (BG, BG_SCALE),
            "BgEvent" => (EVENT, EVENT_SCALE),
            "Sprite" => (SPRITE, SPRITE_SCALE),
            _ => return
        };
        // command arg + (texture > layer > preset)
        f32!(x = (node.arg4.as_deref().or(texture.x.as_deref()).or_else(|| layer.and_then(|l| l.x.as_deref()))), 0.);
        f32!(y = (node.arg5.as_deref().or(texture.y.as_deref()).or_else(|| layer.and_then(|l| l.y.as_deref()))), 0.);
        f32!(z = (texture.z.as_deref()).or_else(|| layer.and_then(|l| l.order.as_deref())), 0.);
        f32!(scale_x = (texture.scale.as_deref()).or_else(|| layer.and_then(|l| l.scale_x.as_deref())), 1.);
        f32!(scale_y = (texture.scale.as_deref()).or_else(|| layer.and_then(|l| l.scale_y.as_deref())), 1.);
        let (off_x, off_y) = (str!(node.arg4), str!(node.arg5));
        commands.spawn((
            Sprite {
                image: asset_server.load(format!("{}{}", img_path, str!(texture.file_name))),
                ..default()
            },
            VNTexture(real_type, str!(node.arg1).into(), str!(node.arg3).into(), scale,
                AvgTransform {
                    orig: Transform::from_xyz(x * scale, y * scale, z)
                            .with_scale(Vec3::new(scale_x * scale, scale_y * scale, 1.)),
                    avg: Transform::from_xyz(f32!(off_x, 0.), f32!(off_y, 0.), z).with_scale(Vec3::ONE)
                }),
            Transform::from_xyz((x + f32!(off_x, 0.)) * scale, (y + f32!(off_y, 0.)) * scale, z)
                .with_scale(Vec3::new(scale_x * scale, scale_y * scale, 1.)),
        ));
    }
}

fn character_off_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    spine_query: &mut Query<(Entity, &mut Spine, &mut VNSpine, &mut Transform), (Without<VNTexture>, Without<VNGui>)>,
    match_label: bool,
) {
    spine_query.iter_mut()
        .filter(|x| {
            match node.arg1.as_deref() {
                None => true,
                // match label name or layer name
                Some(l) => (match_label && x.2.0 == l) || x.2.2 == l,
            }
        }).for_each(|x| {
            info!("remove spine {} with layer {}", x.2.0, x.2.2);
            commands.entity(x.0).despawn()
        }
    )
}

fn bg_off_cmd(
    f: &str,
    commands: &mut Commands,
    tex_query: &mut Query<(Entity, &mut VNTexture, &mut Transform, &mut Sprite), (Without<VNSpine>, Without<VNGui>)>,
) {
    let img_type = match f {
        "BgOff" => TextureType::Bg,
        "BgEventOff" => TextureType::Event,
        _ => return
    };
    tex_query.iter_mut()
        .filter(|x| {
            x.1.0 == img_type
        }).for_each(|(entity, t, _, _)| {
            info!("remove texture {} with layer {}", t.1, t.2);
            commands.entity(entity).despawn();
        }
    )
}

fn sprite_off_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    tex_query: &mut Query<(Entity, &mut VNTexture, &mut Transform, &mut Sprite), (Without<VNSpine>, Without<VNGui>)>,
) {
    tex_query.iter_mut()
        .filter(|x| {
            let type_match = x.1.0 == TextureType::Sprite;
            let label_match = match node.arg1.as_deref() {
                None | Some("AllSpriteObjects") => true,
                // match label name or layer name
                Some(l) => x.1.1 == l || x.1.2 == l,
            };
            type_match && label_match
        }).for_each(|x| {
            info!("remove texture {} with layer {}", x.1.1, x.1.2);
            commands.entity(x.0).despawn();
        }
    )
}

fn layer_off_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    tex_query: &mut Query<(Entity, &mut VNTexture, &mut Transform, &mut Sprite), (Without<VNSpine>, Without<VNGui>)>,
    spine_query: &mut Query<(Entity, &mut Spine, &mut VNSpine, &mut Transform), (Without<VNTexture>, Without<VNGui>)>,
) {
    character_off_cmd(node, commands, spine_query, false);
    tex_query.iter_mut()
        .filter(|x| {
            node.arg1.as_ref().is_none_or(|l| &x.1.2 == l)
        }).for_each(|(entity, t, _, _)| {
            info!("remove texture {} with layer {}", t.1, t.2);
            commands.entity(entity).despawn();
        }
    )
}

fn sound_cmd(
    f: &str,
    node: &utage4::Node,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
    view_res: &ResMut<ViewRes>,
) {
    let sound = match f {
        "BgVoice" => Some(&utage4::SoundEntry{
            file_name: node.voice.clone().map(|v| v.to_lowercase() + ".m4a"),
            ..default()
        }),
        _ => view_res.vn.sound.get(str!(node.arg1)),
    };
    if let Some(sound) = sound {
        f32!(volume = (node.arg3.as_deref()).or(sound.volume.as_deref()), 1.);
        let file = str!(sound.file_name);
        let (audio_path, audio_type, mut loop_type) = match f {
            "Se" => (SE, AudioType::Se, PlaybackMode::Despawn),
            "Bgm" => (BGM, AudioType::Bgm, PlaybackMode::Loop),
            "Ambience" => (AMBIENCE, AudioType::Ambience, PlaybackMode::Loop),
            "HSe" => (SE, AudioType::Se, PlaybackMode::Loop),
            "BgVoice" => (VOICE, AudioType::Se, PlaybackMode::Loop),
            _ => return
        };
        match node.arg2.as_deref() {
            Some("TRUE") => { loop_type = PlaybackMode::Loop }
            Some("FALSE") => { loop_type = PlaybackMode::Despawn }
            _ => ()
        }
        // fade out previous bgm or ambience
        if matches!(audio_type, AudioType::Bgm | AudioType::Ambience) {
            f32!(fade_time = node.arg5, 0.2);
            audio_query.iter_mut()
                .filter(|x| x.2.0 == audio_type)
                .for_each(|(entity, sink, vn)| {
                    info!("fade out {}", vn.1);
                    commands.entity(entity).insert(AudioFade(
                        Timer::from_seconds(fade_time, TimerMode::Once),
                        sink.volume()
                    ));
                }
            )
        }
        info!("play sound {:?}", sound);
        commands.spawn((
            VNAudio(audio_type, str!(node.arg1).into()),
            AudioPlayer::new(
                // replace file extension to m4a
                asset_server.load(format!("{}{}.m4a", audio_path, &file[.. file.len() - 4].to_lowercase()))
            ),
            PlaybackSettings {
                mode: loop_type,
                volume: Volume::Linear(volume),
                ..default()
            },
        ));
    }
}

fn stop_sound_item_cmd(
    f: &str,
    node: &utage4::Node,
    commands: &mut Commands,
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
    ignore_label: bool,
) {
    f32!(fade_time = node.arg6, 0.2);
    let audio_type = match f {
        "StopSe" | "StopHSe" | "StopBgVoice" => Some(AudioType::Se),
        "StopBgm" => Some(AudioType::Bgm),
        "StopAmbience" => Some(AudioType::Ambience),
        _ => None
    };
    audio_query.iter_mut()
        .filter(|x| {
            // none means all type/label
            let type_match = audio_type.as_ref().is_none_or(|t| &x.2.0 == t);
            let label_match = ignore_label || node.arg1.as_ref().is_none_or(|l| &x.2.1 == l);
            type_match && label_match && x.2.0 != AudioType::Voice
        }).for_each(|(entity, sink, vn)| {
            info!("fade out {}", vn.1);
            commands.entity(entity).insert(AudioFade(
                Timer::from_seconds(fade_time, TimerMode::Once),
                sink.volume()
            ));
        }
    )
}

fn stop_sound_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
) {
    let parts = match node.arg1.as_deref() {
        None => vec!["Bgm", "Ambience"],
        Some("All") => vec!["All"],
        Some(s) => s.split(',').collect(),
    };
    if parts.len() > 4 {
        warn!("Ignore weird stop sound command {:?}", parts);
        return
    }
    for p in parts {
        match p {
            "All" => {
                stop_voice_cmd(commands, audio_query);
                stop_sound_item_cmd("", node, commands, audio_query, true);
                return
            }
            "Se" => stop_sound_item_cmd("StopSe", node, commands, audio_query, true),
            "Bgm" => stop_sound_item_cmd("StopBgm", node, commands, audio_query, true),
            "Ambience" => stop_sound_item_cmd("StopAmbience", node, commands, audio_query, true),
            "Voice" => stop_voice_cmd(commands, audio_query),
            _ => (),
        }
    }
}

fn voice_cmd(
    node: &utage4::Node,
    asset_server: &Res<AssetServer>,
    commands: &mut Commands,
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
) {
    if let Some(voice) = &node.voice {
        f32!(volume = node.arg3, 1.);
        let loop_type = match node.arg2.as_deref() {
            Some("TRUE") => PlaybackMode::Loop,
            _ => PlaybackMode::Despawn,
        };
        stop_voice_cmd(commands, audio_query);
        info!("play voice {}", voice);
        commands.spawn((
            VNAudio(AudioType::Voice, "".into()),
            AudioPlayer::new(
                asset_server.load(format!("{}{}.m4a", VOICE, voice.to_lowercase()))
            ),
            PlaybackSettings {
                mode: loop_type,
                volume: Volume::Linear(volume),
                ..default()
            },
        ));
    }
}

fn stop_voice_cmd(
    commands: &mut Commands,
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio), Without<AudioFade>>,
) {
    audio_query.iter_mut().filter(|x| matches!(x.2.0, AudioType::Voice)).for_each(|(entity, _, _)| {
        info!("stop unfinished voice");
        commands.entity(entity).despawn()
    })
}

fn param_cmd(node: &utage4::Node) -> Option<(String, String)> {
    let pattern = str!(node.arg1).replace("\\\"", "");
    if let Some((k, v)) = pattern.split_once('=')
        && !k.is_empty() && !v.is_empty() {
        return Some((k.into(), v.replace('"', "")))
    }
    None
}

fn tween_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    spine_query: &mut Query<(Entity, &mut Spine, &mut VNSpine, &mut Transform), (Without<VNTexture>, Without<VNGui>)>,
    tex_query: &mut Query<(Entity, &mut VNTexture, &mut Transform, &mut Sprite), (Without<VNSpine>, Without<VNGui>)>,
    gui_query: &mut Query<(Entity, &VNGui, &mut Transform), (Without<VNSpine>, Without<VNTexture>)>,
) {
    // MessageWindow = VNGui
    // Graphics = VNSpine + VNTexture
    // Camera = VNGui + VNSpine + VNTexture
    if let Some(t) = Tween::new(node) {
        macro_rules! absxyz {
            ($p:expr) => {
                Vec3::new(
                    t.params.x.unwrap_or($p),
                    t.params.y.unwrap_or($p),
                    t.params.z.unwrap_or($p),
                )
            };
        }
        macro_rules! relxyz {
            ($p:expr) => {
                Vec3::new(
                    t.params.x.unwrap_or($p.x),
                    t.params.y.unwrap_or($p.y),
                    t.params.z.unwrap_or($p.z),
                )
            };
        }
        macro_rules! tween {
            ($lens:ident, $s:ty, $start:expr, $end:expr, $target:expr) => {{
                let tween = bevy_tweening::Tween::new(
                    t.ease_type,
                    t.params.time,
                    $lens { start: $start, end: $end },
                ).with_repeat_count(t.loop_count).with_repeat_strategy(t.loop_type);
                commands.spawn((
                    TweenAnim::new(tween),
                    AnimTarget::component::<$s>($target),
                ))
            }};
        }
        fn calc_color(t: &Tween) -> Color {
            Color::Srgba({
                let mut c = t.params.color.as_deref().and_then(
                    |s| Srgba::hex(s).ok()).unwrap_or(Srgba::WHITE);
                if let Some(v) = t.params.r { c.red = v }
                if let Some(v) = t.params.g { c.green = v }
                if let Some(v) = t.params.b { c.blue = v }
                if let Some(v) = t.params.a { c.alpha = v }
                if let Some(v) = t.params.alpha { c.alpha = v }
                c
            })
        }

        spine_query.iter_mut()
            .filter(|x| ["Graphics", "Camera"].contains(&t.target.as_str()) || t.target == x.2.0)
            .for_each(|mut x| {
                match t.tween_type {
                    TweenType::MoveTo => {
                        let avg_end = relxyz!(x.2.3.avg.translation);
                        let end = x.2.move_to(avg_end, true);
                        tween!(TransformPositionLens, Transform, x.3.translation, end, x.0);
                    },
                    TweenType::MoveFrom => {
                        let avg_start = relxyz!(x.2.3.avg.translation);
                        let start = x.2.move_to(avg_start, false);
                        tween!(TransformPositionLens, Transform, start, x.3.translation, x.0);
                    },
                    TweenType::MoveBy | TweenType::MoveAdd => {
                        let move_by = absxyz!(0.);
                        let end = x.2.move_by(move_by);
                        tween!(TransformPositionLens, Transform, x.3.translation, end, x.0);
                    },
                    TweenType::RotateTo | TweenType::RotateFrom => {
                        let (rx, ry, rz) = x.3.rotation.to_euler(EulerRot::XYZ);
                        let end_x = t.params.x.map(|v| v.to_radians()).unwrap_or(rx);
                        let end_y = t.params.y.map(|v| v.to_radians()).unwrap_or(ry);
                        let end_z = t.params.z.map(|v| v.to_radians()).unwrap_or(rz);
                        let end = Quat::from_euler(EulerRot::XYZ, end_x, end_y, end_z);
                        if t.tween_type == TweenType::RotateTo {
                            tween!(TransformRotationLens, Transform, x.3.rotation, end, x.0);
                        } else {
                            tween!(TransformRotationLens, Transform, end, x.3.rotation, x.0);
                        }
                    },
                    TweenType::RotateBy | TweenType::RotateAdd => {
                        let dx = t.params.x.map(|v| v.to_radians()).unwrap_or(0.);
                        let dy = t.params.y.map(|v| v.to_radians()).unwrap_or(0.);
                        let dz = t.params.z.map(|v| v.to_radians()).unwrap_or(0.);
                        let d = Quat::from_euler(EulerRot::XYZ, dx, dy, dz);
                        tween!(TransformRotationLens, Transform, x.3.rotation, x.3.rotation * d, x.0);
                    },
                    TweenType::ScaleTo => {
                        let avg_end = relxyz!(x.2.3.avg.scale);
                        let end = x.2.scale_to(avg_end, true);
                        tween!(TransformScaleLens, Transform, x.3.scale, end, x.0);
                    },
                    TweenType::ScaleFrom => {
                        let avg_start = relxyz!(x.2.3.avg.scale);
                        let start = x.2.scale_to(avg_start, false);
                        tween!(TransformScaleLens, Transform, start, x.3.scale, x.0);
                    },
                    TweenType::ScaleBy | TweenType::ScaleAdd => {
                        let scale_by = absxyz!(1.);
                        let end = x.2.scale_by(scale_by);
                        tween!(TransformScaleLens, Transform, x.3.scale, end, x.0);
                    },
                    TweenType::ColorTo | TweenType::ColorFrom => {
                        let start = Color::from(Srgba::from_f32_array(x.1.skeleton.get_color()));
                        let end = calc_color(&t);
                        if t.tween_type == TweenType::ColorTo {
                            tween!(SpineColorLens, Spine, start, end, x.0);
                        } else {
                            tween!(SpineColorLens, Spine, end, start, x.0);
                        }
                    },
                    _ => {
                        warn!("Unfinished tween type: {:?} for spine", node.arg2)
                    },
                };
            }
        );

        tex_query.iter_mut()
            .filter(|x| ["Graphics", "Camera"].contains(&t.target.as_str()) || t.target == x.1.1)
            .for_each(|mut x| {
                match t.tween_type {
                    TweenType::MoveTo => {
                        let avg_end = relxyz!(x.1.4.avg.translation);
                        let end = x.1.move_to(avg_end, true);
                        tween!(TransformPositionLens, Transform, x.2.translation, end, x.0);
                    },
                    TweenType::MoveFrom => {
                        let avg_start = relxyz!(x.1.4.avg.translation);
                        let start = x.1.move_to(avg_start, false);
                        tween!(TransformPositionLens, Transform, start, x.2.translation, x.0);
                    },
                    TweenType::MoveBy | TweenType::MoveAdd => {
                        let move_by = absxyz!(0.);
                        let end = x.1.move_by(move_by);
                        tween!(TransformPositionLens, Transform, x.2.translation, end, x.0);
                    },
                    TweenType::RotateTo | TweenType::RotateFrom => {
                        let (rx, ry, rz) = x.2.rotation.to_euler(EulerRot::XYZ);
                        let end_x = t.params.x.map(|v| v.to_radians()).unwrap_or(rx);
                        let end_y = t.params.y.map(|v| v.to_radians()).unwrap_or(ry);
                        let end_z = t.params.z.map(|v| v.to_radians()).unwrap_or(rz);
                        let end = Quat::from_euler(EulerRot::XYZ, end_x, end_y, end_z);
                        if t.tween_type == TweenType::RotateTo {
                            tween!(TransformRotationLens, Transform, x.2.rotation, end, x.0);
                        } else {
                            tween!(TransformRotationLens, Transform, end, x.2.rotation, x.0);
                        }
                    },
                    TweenType::RotateBy | TweenType::RotateAdd => {
                        let dx = t.params.x.map(|v| v.to_radians()).unwrap_or(0.);
                        let dy = t.params.y.map(|v| v.to_radians()).unwrap_or(0.);
                        let dz = t.params.z.map(|v| v.to_radians()).unwrap_or(0.);
                        let d = Quat::from_euler(EulerRot::XYZ, dx, dy, dz);
                        tween!(TransformRotationLens, Transform, x.2.rotation, x.2.rotation * d, x.0);
                    },
                    TweenType::ScaleTo => {
                        let avg_end = relxyz!(x.1.4.avg.scale);
                        let end = x.1.scale_to(avg_end, true);
                        tween!(TransformScaleLens, Transform, x.2.scale, end, x.0);
                    },
                    TweenType::ScaleFrom => {
                        let avg_start = relxyz!(x.1.4.avg.scale);
                        let start = x.1.scale_to(avg_start, false);
                        tween!(TransformScaleLens, Transform, start, x.2.scale, x.0);
                    },
                    TweenType::ScaleBy | TweenType::ScaleAdd => {
                        let scale_by = absxyz!(1.);
                        let end = x.1.scale_by(scale_by);
                        tween!(TransformScaleLens, Transform, x.2.scale, end, x.0);
                    },
                    TweenType::ColorTo | TweenType::ColorFrom => {
                        let end = calc_color(&t);
                        if t.tween_type == TweenType::ColorTo {
                            tween!(SpriteColorLens, Sprite, x.3.color, end, x.0);
                        } else {
                            tween!(SpriteColorLens, Sprite, end, x.3.color, x.0);
                        }
                    },
                    _ => {
                        warn!("Unfinished tween type: {:?} for texture", node.arg2)
                    },
                };
            }
        );

        gui_query.iter_mut()
            .filter(|_| ["MessageWindow", "Camera"].contains(&t.target.as_str()))
            .for_each(|x| {
                match t.tween_type {
                    TweenType::MoveBy => {
                        let move_by = absxyz!(0.);
                        let end = x.2.translation + move_by;
                        tween!(TransformPositionLens, Transform, x.2.translation, end, x.0);
                    },
                    TweenType::RotateBy => {
                        let dx = t.params.x.map(|v| v.to_radians()).unwrap_or(0.);
                        let dy = t.params.y.map(|v| v.to_radians()).unwrap_or(0.);
                        let dz = t.params.z.map(|v| v.to_radians()).unwrap_or(0.);
                        let d = Quat::from_euler(EulerRot::XYZ, dx, dy, dz);
                        tween!(TransformRotationLens, Transform, x.2.rotation, x.2.rotation * d, x.0);
                    },
                    TweenType::ScaleBy => {
                        let scale_by = absxyz!(1.);
                        let end = x.2.scale * scale_by;
                        tween!(TransformScaleLens, Transform, x.2.scale, end, x.0);
                    },
                    _ => {
                        warn!("Unfinished tween type: {:?} for gui", node.arg2)
                    },
                };
            }
        );
    } else {
        warn!("Unimplemented tween type: {:?}", node.arg2);
    }
}
