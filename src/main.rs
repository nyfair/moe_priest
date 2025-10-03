#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utage4;

use bevy::audio::PlaybackMode;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, CoreScrollbarThumb, Scrollbar, ScrollbarPlugin};
use bevy::window::{PrimaryWindow, WindowMode};
use bevy_spine::prelude::*;
use bevy_transform_interpolation::prelude::*;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use std::time::Duration;

use crate::utage4::VNConfig;

static FONT: &str = "FOT-NewRodinProN-EB.otf";
static HEADTEXT: Color = Color::srgb(0.5, 0.8, 0.7);
static LISTTEXT: Color = Color::srgb(0.2, 0.8, 0.2);
static SELECTTEXT: Color = Color::srgb(0.8, 0.8, 0.8);
static HOVERBG: Color = Color::srgb(0.1, 0.4, 0.1);
static VNSPEED: Duration = Duration::from_millis(100);

#[derive(Debug)]
struct Location {
    path: String,
    name: String,
    ext: String,
}

#[derive(Clone, Debug, PartialEq)]
enum ListMode {
    Gallery,
    Motion,
    Memory,
}

#[derive(Resource)]
struct Viewres {
    scene_menu: Option<Entity>,
    anime_menu: Option<Entity>,
    cur_spine: Vec<Entity>,
    spines: BTreeMap<String, Location>,
    events: BTreeMap<String, Location>,
    cur_mode: ListMode,
    vn: VNConfig,
}

#[derive(Component)]
struct SceneMenu;

#[derive(Component)]
struct AnimeMenu;

#[derive(Component)]
struct ModeMenu;

#[derive(Component)]
struct VNText {
    text: String,
    index: usize,
    timer: Timer,
}

impl VNText {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            index: 0,
            timer: Timer::new(VNSPEED, TimerMode::Once),
        }
    }
}

#[derive(Message)]
struct SceneMsg(ListMode);

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
        .insert_resource(Time::<Fixed>::from_hz(5.))
        .add_message::<SceneMsg>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            list_scene,
            choose_scene,
            spine_spawn.in_set(SpineSet::OnReady),
            choose_animation,
            choose_mode,
            hide_ui,
            vn_dialogue,
        ))
        .add_systems(FixedUpdate, (scroll, mouse_motion))
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
            if let (Some(l), Some(r)) = (event.rfind('/'), event.find('.'))
                && l < r {
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
    commands.insert_resource(Viewres {
        scene_menu: None,
        anime_menu: None,
        cur_spine: vec![],
        spines,
        events,
        cur_mode: ListMode::Gallery,
        vn,
    });

    commands.spawn((
        Visibility::Visible,
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
            asset_server.load("advscene/resources/advscene/sound/voice/ch_30005/general/basic/30005_030.m4a")
        ),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    ));
}

fn list_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut view_res: ResMut<Viewres>,
    mut scene_msg: MessageReader<SceneMsg>,
) {
    if let Some(event) = scene_msg.read().last() {
        if let Some(entity) = view_res.scene_menu {
            commands.entity(entity).despawn();
            view_res.scene_menu = None;
        }
        let menu = commands.spawn((
            Visibility::Visible,
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
        }).id();
        view_res.scene_menu = Some(menu);
    }
}

fn choose_scene(
    asset_server: Res<AssetServer>,
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut TextColor,
        &mut BackgroundColor,
        &SceneMenu,
    ), (Changed<Interaction>, With<Button>),>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut view_res: ResMut<Viewres>,
) {
    for (interaction, text, mut color, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let bundle_name = &text.to_string();
                if let Some(file) = view_res.spines.get(bundle_name) {
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
                    for spine in view_res.cur_spine.iter() {
                        commands.entity(*spine).despawn();
                    }
                    let spine = commands.spawn(SpineBundle {
                        skeleton: skeleton_handle.clone().into(),
                        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::ONE * 0.5),
                        ..Default::default()
                    }).id();
                    view_res.cur_spine = vec![spine];
                }
            },
            Interaction::Hovered => {
                *color = SELECTTEXT.into();
                *bg_color = HOVERBG.into();
            },
            _ => {
                *color = LISTTEXT.into();
                *bg_color = Color::NONE.into();
            }
        }
    }
}

fn spine_spawn(
    asset_server: Res<AssetServer>,
    mut spine_ready_msg: MessageReader<SpineReadyMsg>,
    mut spine_query: Query<&mut Spine>,
    mut commands: Commands,
    mut view_res: ResMut<Viewres>,
) {
    for msg in spine_ready_msg.read() {
        if let Some(entity) = view_res.anime_menu {
            commands.entity(entity).despawn();
            view_res.anime_menu = None;
        }
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

        let menu = commands.spawn((
            Visibility::Visible,
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
        }).id();
        view_res.anime_menu = Some(menu);
    };
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
    for (interaction, text, mut color, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                for mut spine in &mut spine_query {
                    let _ = spine.animation_state.set_animation_by_name(0, text, true);
                }
            },
            Interaction::Hovered => {
                *color = SELECTTEXT.into();
                *bg_color = HOVERBG.into();
            },
            _ => {
                *color = LISTTEXT.into();
                *bg_color = Color::NONE.into();
            }
        }
    }
}

fn choose_mode(
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut BackgroundColor,
        &ModeMenu,
    ), (Changed<Interaction>, With<Button>),>,
    mut view_res: ResMut<Viewres>,
    mut scene_msg: MessageWriter<SceneMsg>,
) {
    for (interaction, text, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mode = match text.as_str() {
                    "Motion" => ListMode::Motion,
                    "Memory" => ListMode::Memory,
                    _ => ListMode::Gallery,
                };
                view_res.cur_mode = mode.clone();
                scene_msg.write(SceneMsg(mode));
            },
            Interaction::Hovered => {
                *bg_color = HOVERBG.into();
            },
            _ => {
                *bg_color = Color::NONE.into();
            }
        }
    }
}

fn hide_ui(
    mut ui: Query<&mut Visibility>,
    button: Res<ButtonInput<MouseButton>>,
) {
    if button.just_released(MouseButton::Right) {
        for mut v in &mut ui {
            v.toggle_visible_hidden();
        }
    }
}

fn scroll(
    mut query: Query<&mut Transform, With<Spine>>,
    mut scroll: MessageReader<MouseWheel>,
    scrollbar_query: Query<&Scrollbar>,
    mut scrolled_query: Query<(&mut ScrollPosition, &ComputedNode), Without<Scrollbar>>,
    window: Single<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    for ev in scroll.read() {
        if ev.y == 0. {
            break
        }
        let delta_secs = time.delta_secs();
        if let Some(pos) = window.cursor_position() {
            if pos.x > window.width() * 0.88 {
                for scrollbar in scrollbar_query {
                    if let Ok((mut scroll_pos, scroll_content)) = scrolled_query.get_mut(scrollbar.target) {
                        let visible_size = scroll_content.size() * scroll_content.inverse_scale_factor;
                        let content_size = scroll_content.content_size() * scroll_content.inverse_scale_factor;
                        let range = (content_size.y - visible_size.y).max(0.);
                        scroll_pos.y -= ev.y * 5000. * delta_secs;
                        scroll_pos.y = scroll_pos.y.clamp(0., range);
                    };
                }
            } else {
                for mut spine in &mut query {
                    spine.scale += ev.y * 0.1 * delta_secs;
                }
            }
        }
    }
}

fn mouse_motion(
    mut query: Query<&mut Transform, With<Spine>>,
    mut motion: MessageReader<MouseMotion>,
    button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if button.pressed(MouseButton::Middle) {
        let delta_secs = time.delta_secs();
        for ev in motion.read() {
            for mut spine in &mut query {
                spine.translation.x += ev.delta.x * 6. * delta_secs;
                spine.translation.y -= ev.delta.y * 6. * delta_secs;
            }
        }
    }
}

fn vn_dialogue(
    mut query: Query<&mut VNText>,
    e_query: Query<Entity, With<VNText>>,
    view_res: Res<Viewres>,
    time: Res<Time>,
    mut writer: TextUiWriter,
) {
    if view_res.cur_mode == ListMode::Memory {
        for mut say in query.iter_mut() {
            say.timer.tick(time.delta());
            if say.timer.just_finished() && say.index < say.text.len() {
                say.index += 1;
                let displayed_text: String = say.text
                    .chars()
                    .take(say.index)
                    .collect();
                for entity in e_query.iter() {
                    *writer.text(entity, 0)  = displayed_text.clone();
                }
            }
        }
    }
}
