#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{image::ImageSampler, prelude::*, render::render_resource::{FilterMode, SamplerDescriptor}, window};
use bevy_spine::{prelude::*, textures::SpineTextureCreateEvent};
use std::fs::read_to_string;

static FONT: &str = "resources/font/FOT-KurokaneStd-EB.otf";

#[derive(Resource)]
struct Viewres {
    scene_menu: Option<Entity>,
    cur_spine: Vec<Entity>,
}

#[derive(Component)]
struct SceneMenu;

#[derive(Component)]
struct SpineMenu;

#[derive(Default)]
struct SetSpineTextureNearest {
    handles: Vec<Handle<Image>>,
}

fn set_spine_texture_nearest(
    mut local: Local<SetSpineTextureNearest>,
    mut spine_texture_create_events: EventReader<SpineTextureCreateEvent>,
    mut images: ResMut<Assets<Image>>,
) {
    for spine_texture_create_event in spine_texture_create_events.read() {
        local
            .handles
            .push(spine_texture_create_event.handle.clone());
    }
    let mut removed_handles = vec![];
    for (handle_index, handle) in local.handles.iter().enumerate() {
        if let Some(image) = images.get_mut(handle) {
            image.sampler = ImageSampler::Descriptor(SamplerDescriptor {
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            }.into());
            removed_handles.push(handle_index);
        }
    }
    for removed_handle in removed_handles.into_iter().rev() {
        local.handles.remove(removed_handle);
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    resizable: false,
                    mode: window::WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    title: "Spine".to_string(),
                    ..default()
                }),
                ..default()
            }
        ),
            SpinePlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            choose_spine,
            spine_spawn.in_set(SpineSet::OnReady),
            set_spine_texture_nearest.after(SpineSystem::Load),
            choose_animation,
            hide_ui
        ))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    commands.spawn(Camera2d::default());
    commands.insert_resource(Viewres {
        scene_menu: None,
        cur_spine: vec![],
    });
    commands.spawn((
        Visibility::Visible,
        Node {
            width: Val::Percent(11.),
            height: Val::Percent(96.),
            left: Val::Percent(88.),
            top: Val::Percent(1.),
            align_items: AlignItems::End,
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new("Select Character"),
            TextFont {
                font: asset_server.load(FONT),
                font_size: 20.,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.8, 0.7)),
            TextLayout::new_with_justify(JustifyText::Right),
        ));
        for spine in read_to_string("assets/binary_spine.txt").unwrap().lines() {
            parent.spawn((
                Button,
                SpineMenu,
                Node {
                    width: Val::Percent(90.),
                    ..default()
                },
                Text::new(spine.to_string()),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 20.,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.8, 0.2)),
                BackgroundColor(Color::srgb(0., 0., 0.)),
                TextLayout::new_with_justify(JustifyText::Right),
            ));
        }
    });
}

fn choose_spine(
    asset_server: Res<AssetServer>,
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut TextColor,
        &mut BackgroundColor,
        &SpineMenu,
    ), (Changed<Interaction>, With<Button>),>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
    mut view_res: ResMut<Viewres>,
) {
    for (interaction, text, mut color, mut bg_color, _) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let skeleton = SkeletonData::new_from_binary(
                    asset_server.load(format!("spine/{}.skel.bytes", text.to_string())),
                    asset_server.load(format!("spine/{}.atlas.txt", text.to_string())),
                );
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
            },
            Interaction::Hovered => {
                *color = Color::srgb(0.8, 0.8, 0.8).into();
                *bg_color = Color::srgb(0.1, 0.4, 0.1).into();
            },
            _ => {
                *color = Color::srgb(0.2, 0.8, 0.2).into();
                *bg_color = Color::srgb(0., 0., 0.).into();
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
        if let Some(entity) = view_res.scene_menu {
            commands.entity(entity).despawn();
            view_res.scene_menu = None;
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
                height: Val::Percent(96.),
                left: Val::Percent(1.),
                top: Val::Percent(1.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Select Animation"),
                TextFont {
                    font: asset_server.load(FONT),
                    font_size: 20.,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.8, 0.7)),
            ));
            for animation in animation_list {
                parent.spawn((
                    Button,
                    SceneMenu,
                    Node {
                        width: Val::Percent(90.),
                        ..default()
                    },
                    Text::new(animation),
                    TextFont {
                        font: asset_server.load(FONT),
                        font_size: 20.,
                        ..default()
                    },
                    TextColor(Color::srgb(0.2, 0.8, 0.2)),
                    BackgroundColor(Color::srgb(0., 0., 0.)),
                ));
            }
        }).id();
        view_res.scene_menu = Some(menu);
    };
}

fn choose_animation(
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut TextColor,
        &mut BackgroundColor,
        &SceneMenu,
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
                *color = Color::srgb(0.8, 0.8, 0.8).into();
                *bg_color = Color::srgb(0.1, 0.4, 0.1).into();
            },
            _ => {
                *color = Color::srgb(0.2, 0.8, 0.2).into();
                *bg_color = Color::srgb(0., 0., 0.).into();
            }
        }
    }
}

fn hide_ui(mut query: Query<&mut Visibility>, button: Res<ButtonInput<MouseButton>>) {
    if button.just_released(MouseButton::Right) {
        for mut v in &mut query {
            v.toggle_visible_hidden();
        }
    }
}
