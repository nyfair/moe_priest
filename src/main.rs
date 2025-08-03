#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utage4;

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode};
use bevy_spine::prelude::*;
use bevy_transform_interpolation::prelude::*;
use std::collections::BTreeMap;
use std::fs::read_to_string;

static FONT: &str = "resources/font/FOT-KurokaneStd-EB.otf";
static HEADTEXT: Color = Color::srgb(0.5, 0.8, 0.7);
static LISTTEXT: Color = Color::srgb(0.2, 0.8, 0.2);
static SELECTTEXT: Color = Color::srgb(0.8, 0.8, 0.8);
static HOVERBG: Color = Color::srgb(0.1, 0.4, 0.1);

#[derive(Debug)]
struct Location {
    path: String,
    name: String,
    ext: String,
}

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
}

#[derive(Component)]
struct SceneMenu;

#[derive(Component)]
struct AnimeMenu;

#[derive(Component)]
struct ModeMenu;

#[derive(Event)]
struct SceneEvent(ListMode);

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
        // ).set(
        //     bevy::render::RenderPlugin {
        //         render_creation: bevy::render::settings::WgpuSettings {
        //             backends: Some(bevy::render::settings::Backends::DX12),
        //             ..default()
        //         }
        //         .into(),
        //         ..default()
        //     }
        ),
            SpinePlugin,
            TransformInterpolationPlugin::interpolate_all(),
        ))
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(Time::<Fixed>::from_hz(5.))
        .add_event::<SceneEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            list_scene,
            choose_scene,
            spine_spawn.in_set(SpineSet::OnReady),
            choose_animation,
            choose_mode,
            hide_ui,
        ))
        .add_systems(FixedUpdate, (scroll, mouse_motion))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut event: EventWriter<SceneEvent>,
) {
    if let Ok(content) = read_to_string("assets/advscene/scenariochapter/config.chapter.json") {
        let _vn = utage4::parse_chapter(content).unwrap();
    }
    
    let mut spines = BTreeMap::new();
    if let Ok(content) = read_to_string("assets/spine.txt") {
        for spine in content.lines() {
            if let (Some(l), Some(r)) = (spine.rfind('/'), spine.rfind('.')) {
                if l < r {
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
    }
    let mut events = BTreeMap::new();
    if let Ok(content) = read_to_string("assets/event.txt") {
        for event in content.lines() {
            if let (Some(l), Some(r)) = (event.rfind('/'), event.find('.')) {
                if l < r {
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
    }

    commands.spawn(Camera2d);
    commands.insert_resource(Viewres {
        scene_menu: None,
        anime_menu: None,
        cur_spine: vec![],
        spines,
        events,
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
    event.write(SceneEvent(ListMode::Gallery));
}

fn list_scene(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut view_res: ResMut<Viewres>,
    mut events: EventReader<SceneEvent>,
) {
    if let Some(event) = events.read().last() {
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
                TextLayout::new_with_justify(JustifyText::Right),
            ));
            parent.spawn((
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
                        TextLayout::new_with_justify(JustifyText::Right),
                    ));
                }
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
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
    mut commands: Commands,
    mut view_res: ResMut<Viewres>,
) {
    for event in spine_ready_event.read() {
        if let Some(entity) = view_res.anime_menu {
            commands.entity(entity).despawn();
            view_res.anime_menu = None;
        }
        let mut animation_list = Vec::new();
        if let Ok(mut spine) = spine_query.get_mut(event.entity) {
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
    mut event: EventWriter<SceneEvent>,
) {
    for (interaction, text, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mode = match text.as_str() {
                    "Motion" => ListMode::Motion,
                    "Memory" => ListMode::Memory,
                    _ => ListMode::Gallery,
                };
                event.write(SceneEvent(mode));
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
    if button.just_released(MouseButton::Middle) {
        for mut v in &mut ui {
            v.toggle_visible_hidden();
        }
    }
}

fn scroll(
    mut query: Query<&mut Transform, With<Spine>>,
    mut scroll: EventReader<MouseWheel>,
    mut scrolled_query: Query<&mut ScrollPosition>,
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
                for mut scroll_position in &mut scrolled_query {
                    scroll_position.offset_y -= ev.y * 5000. * delta_secs;
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
    mut motion: EventReader<MouseMotion>,
    button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    if button.pressed(MouseButton::Right) {
        let delta_secs = time.delta_secs();
        for ev in motion.read() {
            for mut spine in &mut query {
                spine.translation.x += ev.delta.x * 6. * delta_secs;
                spine.translation.y -= ev.delta.y * 6. * delta_secs;
            }
        }
    }
}
