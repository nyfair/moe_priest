#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utage4;

use bevy::audio::{PlaybackMode, Volume};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar, ScrollbarPlugin};
use bevy::window::{PrimaryWindow, WindowMode};
use bevy_spine::prelude::*;
use bevy_transform_interpolation::prelude::*;
use regex::{Regex, Captures};
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::time::Duration;

use crate::utage4::VNConfig;

const FONT: &str = "FOT-NewRodinProN-EB.otf";
const HEADTEXT: Color = Color::srgb(0.5, 0.8, 0.7);
const LISTTEXT: Color = Color::srgb(0.2, 0.8, 0.2);
const SELECTTEXT: Color = Color::srgb(0.8, 0.8, 0.8);
const HOVERBG: Color = Color::srgb(0.1, 0.4, 0.1);
const VNSPEED: Duration = Duration::from_millis(60);
const CHARTEXT: Color = Color::srgb_u8(237, 221, 192);
const VNTEXT: Color = Color::srgb_u8(78, 72, 70);
const Z_CG: i32 = 300;
const Z_UI: i32 = 993;
const Z_TEXT: i32 = 996;
const Z_FADE: i32 = 999;

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
    wait_timer: Option<Timer>,
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
        self.text = text.to_string();
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
struct VNTexture(TextureType, String, String);

#[derive(PartialEq)]
enum AudioType {
    Bgm,
    Se,
    Ambience,
    Voice,
}

#[derive(Component)]
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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    ..default()
                }),
                ..default()
            }
        ),
            ScrollbarPlugin,
            SpinePlugin,
            TransformInterpolationPlugin::interpolate_all(),
        ))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Time::<Fixed>::from_hz(10.))
        .add_message::<SceneMsg>()
        .add_message::<VNToogleMsg>()
        .add_message::<VNMsg>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            list_scene,
            choose_scene,
            spine_spawn.in_set(SpineSet::OnReady),
            choose_animation,
            choose_mode,
            input_handler,
            toggle_vn,
            play_vn,
            vn_dialogue,
            fade_overlay,
            fade_sound,
            check_wait,
        ))
        .add_systems(FixedUpdate, (mouse_scroll, mouse_spine_move))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut scene_msg: MessageWriter<SceneMsg>,
) {
    let vn = if let Ok(content) = read_to_string("assets/advscene/scenariochapter/config.chapter.json") {
        utage4::parse_chapter(content)
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

    commands.spawn(Camera2d);
    commands.insert_resource(ViewRes {
        spines,
        events,
        mode: ListMode::Gallery,
        vn,
        avg: false,
        avg_nodes: Vec::new(),
        avg_offset: 0,
        // <interval=???> to ..., remove other tags
        avg_regex: Regex::new(r"(?P<interval_tag><interval=[^>]*>)|(?P<other_tag><[^>]*>)").unwrap(),
        fast: false,
        wait_timer: None,
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
                    font_size: 24.,
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
            asset_server.load(format!("{}ch_30005/general/basic/30005_030.m4a", VOICE))
        ),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));

    commands.spawn((
        Visibility::Hidden,
        Sprite {
            image: asset_server.load("AdvScene.png"),
            color: Color::srgba(1., 1., 1., 0.6),
            ..default()
        },
        VNGui,
        Transform::from_translation(Vec3::new(0., -457., Z_UI as f32)).with_scale(Vec3::ONE),
    ));
    commands.spawn((
        Visibility::Hidden,
        ZIndex(Z_TEXT),
        Node {
            top: Val::Percent(78.),
            left: Val::Percent(24.),
            ..default()
        },
        Text::new(""),
        VNGui,
        VNChar,
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.,
            ..default()
        },
        TextColor(CHARTEXT),
    ));
    commands.spawn((
        Visibility::Hidden,
        ZIndex(Z_TEXT),
        Node {
            top: Val::Percent(84.),
            left: Val::Percent(25.),
            ..default()
        },
        Text::new(""),
        VNGui,
        VNText::new(),
        TextFont {
            font: asset_server.load(FONT),
            font_size: 32.,
            ..default()
        },
        TextColor(VNTEXT),
    ));
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
                    font_size: 24.,
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
                                font_size: 20.,
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
                    commands.spawn(SpineBundle {
                        skeleton: skeleton_handle.clone().into(),
                        transform: Transform::from_xyz(0., 0., Z_CG as f32).with_scale(Vec3::ONE * 0.5),
                        ..Default::default()
                    });
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
    })
}

fn spine_spawn(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut spine_query: Query<&mut Spine>,
    anime_query: Query<Entity, With<AnimeMenuList>>,
    mut spine_ready_msg: MessageReader<SpineReadyMsg>,
    view_res: Res<ViewRes>,
) {
    if !view_res.avg {
        for msg in spine_ready_msg.read() {
            anime_query.iter().for_each(|entity| {
                commands.entity(entity).despawn()
            });
            let mut animation_list = Vec::new();
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
                        font_size: 24.,
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
                            font_size: 20.,
                            ..default()
                        },
                        TextColor(LISTTEXT),
                        BackgroundColor(Color::NONE),
                    ));
                }
            });
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
    mut spine_query: Query<&mut Spine>,
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
        | key.just_pressed(KeyCode::Enter) | key.just_pressed(KeyCode::Space) {
            vn_msg.write(VNMsg);
        }
        if key.just_pressed(KeyCode::Escape) {
            view_res.avg = false;
            view_res.wait_timer = None;
            vn_ui_msg.write(VNToogleMsg(false));
        }
        if key.just_released(KeyCode::ControlLeft) | key.just_released(KeyCode::ControlRight) {
            view_res.fast = false;
        }
        if key.pressed(KeyCode::ControlLeft) | key.pressed(KeyCode::ControlRight) {
            view_res.fast = true;
        }
    }
}

fn toggle_vn(
    mut commands: Commands,
    mut viewer_ui: Query<&mut Visibility, Without<VNGui>>,
    mut vn_ui: Query<&mut Visibility, With<VNGui>>,
    mut text: Single<&mut Text, With<VNText>>,
    mut vn_text: Single<&mut VNText>,
    spine_query: Query<Entity, With<Spine>>,
    despawn_query: Query<Entity, Or<(With<FadeOverlay>, With<VNTexture>, With<VNAudio>)>>,
    mut vn_ui_msg: MessageReader<VNToogleMsg>,
    mut vn_msg: MessageWriter<VNMsg>,
) {
    if let Some(msg) = vn_ui_msg.read().last() {
        spine_query.iter().for_each(|entity| {
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
            despawn_query.iter().for_each(|entity| {
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

fn mouse_spine_move(
    mut spine_query: Query<&mut Transform, Or<(With<Spine>, With<VNTexture>)>>,
    mut motion: MessageReader<MouseMotion>,
    button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if button.pressed(MouseButton::Middle) {
        let delta_secs = time.delta_secs();
        for ev in motion.read() {
            spine_query.iter_mut().for_each(|mut spine| {
                spine.translation.x += ev.delta.x * 6. * delta_secs;
                spine.translation.y -= ev.delta.y * 6. * delta_secs;
            })
        }
    }
}

fn check_wait(
    mut vn_msg: MessageWriter<VNMsg>,
    time: Res<Time>,
    mut view_res: ResMut<ViewRes>,
) {
    if view_res.avg {
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

fn vn_dialogue(
    mut vn_text: Single<&mut VNText>,
    mut text: Single<&mut Text, With<VNText>>,
    fade_query: Query<&FadeOverlay>,
    time: Res<Time>,
    view_res: Res<ViewRes>,
) {
    if view_res.avg && fade_query.count() == 0 {
        vn_text.timer.tick(time.delta());
        if vn_text.timer.just_finished() && vn_text.index < vn_text.len() {
            vn_text.index += 1;
            let displayed_text: String = vn_text.text
                .chars()
                .take(vn_text.index)
                .collect();
            text.0 = displayed_text.clone();
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

fn play_vn(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut vn_char: Single<&mut Text, With<VNChar>>,
    mut vn_text: Single<&mut VNText>,
    mut vn_ui: Query<&mut Visibility, With<VNGui>>,
    mut texture_query: Query<(Entity, &VNTexture)>,
    mut audio_query: Query<(Entity, &AudioSink, &VNAudio)>,
    mut vn_msg: MessageReader<VNMsg>,
    mut vn_ui_msg: MessageWriter<VNToogleMsg>,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut view_res: ResMut<ViewRes>,
) {
    if vn_msg.read().last().is_some() {
        if !view_res.fast && let Some(_) = &view_res.wait_timer {
            return
        }
        if vn_text.finished() {
            while view_res.avg_offset < view_res.avg_nodes.len() {
                let node = &view_res.avg_nodes[view_res.avg_offset];
                info!("{:?}", node);
                match node.command.as_ref().map(|s| &s[..]) {
                    None => {
                        let char_name = str!(node.arg1);
                        vn_char.0 = char_name.to_string();
                        if let Some(t) = &node.text {
                            let text = view_res.avg_regex.replace_all(t, |caps: &Captures| {
                                if caps.name("interval_tag").is_some() {
                                    "……"
                                } else {
                                    ""
                                }
                            }).into_owned();
                            vn_text.update(&text);
                            vn_ui.iter_mut().for_each(|mut v| {
                                *v = Visibility::Visible
                            });
                        }
                        if let Some(character) = view_res.vn.character.get(char_name) {
                            info!("load chara {:?}", character);
                            let layer = view_res.vn.layer.get(str!(node.arg3));
                            // command arg > character > layer > preset
                            f32!(x = (node.arg4.as_deref().or(character.x.as_deref())
                                .or_else(|| layer.and_then(|l| l.x.as_deref()))), 0.);
                            f32!(y = (node.arg5.as_deref().or(character.y.as_deref())
                                .or_else(|| layer.and_then(|l| l.y.as_deref()))), 0.);
                            f32!(z = (character.z.as_deref())
                                .or_else(|| layer.and_then(|l| l.order.as_deref())), 0.);
                            f32!(scale_x = (character.scale.as_deref())
                                .or_else(|| layer.and_then(|l| l.scale_x.as_deref())), 0.5);
                            f32!(scale_y = (character.scale.as_deref())
                                .or_else(|| layer.and_then(|l| l.scale_y.as_deref())), 0.5);
                            let file_name = str!(character.file_name);
                            if let (Some(l), Some(r)) = (file_name.rfind('/'), file_name.rfind('.')) && l < r {
                                let path = file_name[..l].to_string();
                                if let Some(rr) = path.rfind('/') {
                                    let bundle_name = path[rr+1..].to_string();
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
                                        commands.spawn(SpineBundle {
                                            skeleton: skeleton_handle.clone().into(),
                                            transform: Transform::from_xyz(x, y, z as f32).
                                                with_scale(Vec3::new(scale_x, scale_y, 1.)),
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                        if let Some(voice) = &node.voice {
                            stop_voice_cmd(&mut commands, &mut audio_query);
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
                        view_res.avg_offset += 1;
                        break;
                    }
                    Some(f @ "FadeOut") | Some(f @ "FadeIn") => {
                        fade_overlay_cmd(f, node, &mut commands);
                    }
                    Some(f @ "Bg") | Some(f @ "BgEvent") | Some(f @ "Sprite") => {
                        img_cmd(f, node, &asset_server, &mut commands, &view_res);
                    }
                    Some(f @ "BgOff") | Some(f @ "BgEventOff") => {
                        bg_off_cmd(f, &mut commands, &mut texture_query);
                    }
                    Some("SpriteOff") => {
                        sprite_off_cmd(node, &mut commands, &mut texture_query);
                    }
                    Some("LayerOff") => {
                        layer_off_cmd(node, &mut commands, &mut texture_query);
                    }
                    Some(f @ "Se") | Some(f @ "Bgm") | Some(f @ "Ambience") => {
                        sound_cmd(f, node, &asset_server, &mut commands, &mut audio_query, &view_res);
                    }
                    Some(f @ "StopSe") | Some(f @ "StopBgm") | Some(f @ "StopAmbience") => {
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
            vn_text.skip_to_end();
            vn_ui.iter_mut().for_each(|mut v| {
                *v = Visibility::Visible
            })
        }
    }
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
        let (img_path, scale_factor) = match texture_type {
            "Bg" => (BG, 1.725),
            "BgEvent" => (EVENT, 1.35),
            "Sprite" => (SPRITE, 1.),
            _ => return
        };
        // command arg > texture > layer > preset
        f32!(x = (node.arg4.as_deref().or(texture.x.as_deref()).or_else(|| layer.and_then(|l| l.x.as_deref()))), 0.);
        f32!(y = (node.arg5.as_deref().or(texture.y.as_deref()).or_else(|| layer.and_then(|l| l.y.as_deref()))), 0.);
        f32!(z = (texture.z.as_deref()).or_else(|| layer.and_then(|l| l.order.as_deref())), 0.);
        f32!(scale_x = (texture.scale.as_deref()).or_else(|| layer.and_then(|l| l.scale_x.as_deref())), 1.);
        f32!(scale_y = (texture.scale.as_deref()).or_else(|| layer.and_then(|l| l.scale_y.as_deref())), 1.);
        commands.spawn((
            Sprite {
                image: asset_server.load(format!("{}{}", img_path, str!(texture.file_name))),
                ..default()
            },
            VNTexture(real_type, str!(node.arg1).into(), str!(node.arg3).into()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::new(scale_x * scale_factor, scale_y * scale_factor, 1.)),
        ));
    }
}

fn bg_off_cmd(
    f: &str,
    commands: &mut Commands,
    texture_query: &mut Query<(Entity, &VNTexture)>,
) {
    let img_type = match f {
        "BgOff" => TextureType::Bg,
        "BgEventOff" => TextureType::Event,
        _ => return
    };
    texture_query.iter_mut()
        .filter(|x| {
            x.1.0 == img_type
        }).for_each(|(entity, t)| {
            info!("remove texture {} with layer {}", t.1, t.2);
            commands.entity(entity).despawn();
        }
    )
}

fn sprite_off_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    texture_query: &mut Query<(Entity, &VNTexture)>,
) {
    texture_query.iter_mut()
        .filter(|x| {
            let type_match = x.1.0 == TextureType::Sprite;
            let label_match = match node.arg1.as_deref() {
                None | Some("AllSpriteObjects") => true,
                // match label name or layer name
                Some(label) => x.1.1 == label || x.1.2 == label,
            };
            type_match && label_match
        }).for_each(|(entity, t)| {
            info!("remove texture {} with layer {}", t.1, t.2);
            commands.entity(entity).despawn();
        }
    )
}

fn layer_off_cmd(
    node: &utage4::Node,
    commands: &mut Commands,
    texture_query: &mut Query<(Entity, &VNTexture)>,
) {
    texture_query.iter_mut()
        .filter(|x| {
            node.arg1.as_ref().is_none_or(|l| &x.1.2 == l)
        }).for_each(|(entity, t)| {
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
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio)>,
    view_res: &ResMut<ViewRes>,
) {
    let sound = view_res.vn.sound.get(str!(node.arg1));
    if let Some(sound) = sound {
        f32!(volume = (node.arg3.as_deref()).or(sound.volume.as_deref()), 1.);
        let file = str!(sound.file_name);
        let (audio_path, audio_type, mut loop_type) = match f {
            "Se" => (SE, AudioType::Se, PlaybackMode::Despawn),
            "Bgm" => (BGM, AudioType::Bgm, PlaybackMode::Loop),
            "Ambience" => (SE, AudioType::Ambience, PlaybackMode::Loop),
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
                asset_server.load(format!("{}{}.m4a", audio_path, &file[.. file.len() - 4]))
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
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio)>,
    ignore_label: bool,
) {
    f32!(fade_time = node.arg6, 0.2);
    let audio_type = match f {
        "StopSe" => Some(AudioType::Se),
        "StopBgm" => Some(AudioType::Bgm),
        "StopAmbience" => Some(AudioType::Ambience),
        _ => None
    };
    audio_query.iter_mut()
        .filter(|x| {
            // none means all type/label
            let type_match = audio_type.as_ref().is_none_or(|t| &x.2.0 == t);
            let label_match = ignore_label | node.arg1.as_ref().is_none_or(|l| &x.2.1 == l);
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
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio)>,
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
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio)>,
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
                asset_server.load(format!("{}{}.m4a", VOICE, voice))
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
    audio_query: &mut Query<(Entity, &AudioSink, &VNAudio)>,
) {
    audio_query.iter_mut().filter(|x| matches!(x.2.0, AudioType::Voice)).for_each(|(entity, _, _)| {
        info!("stop unfinished voice");
        commands.entity(entity).despawn()
    })
}
