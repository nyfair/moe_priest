#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    ecs::{system::StaticSystemParam}, prelude::*, window, winit::WinitSettings
};
use bevy_spine::{
    materials::{SpineMaterial, SpineMaterialInfo, SpineMaterialPlugin, SpineSettingsQuery},
    prelude::*,
    SpineMeshType,
};

#[derive(Component)]
struct Spine3DMaterial;

impl SpineMaterial for Spine3DMaterial {
    type MeshMaterial = MeshMaterial3d<StandardMaterial>;
    type Material = StandardMaterial;
    type Params<'w, 's> = SpineSettingsQuery<'w, 's>;

    fn update(
        material: Option<Self::Material>,
        _: Entity,
        renderable_data: SpineMaterialInfo,
        _: &StaticSystemParam<Self::Params<'_, '_>>,
    ) -> Option<Self::Material> {
        let mut material = material.unwrap_or_else(|| Self::Material {
            unlit: true,
            alpha_mode:AlphaMode::Premultiplied,
            ..Self::Material::default()
        });
        material.base_color_texture = Some(renderable_data.texture);
        Some(material)
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
            SpineMaterialPlugin::<Spine3DMaterial>::default(),
        ))
        .insert_resource(WinitSettings::game())
        .insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
        .add_systems(Startup, setup)
        .add_systems(Update, on_spawn.in_set(SpineSet::OnReady))
        .add_systems(Update, choose_animation)
        .add_systems(Update, hide_ui)
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut skeletons: ResMut<Assets<SkeletonData>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            90.0_f32.to_radians().cos() * 8.5,
            10.0_f32.to_radians().tan() * 8.5,
            90.0_f32.to_radians().sin() * 8.5,
        ).looking_at(Vec3::new(0., 1.5, 0.), Vec3::Y),
    ));

    let skeleton = SkeletonData::new_from_binary(
        asset_server.load("rev66001_01.skel.bytes"),
        asset_server.load("rev66001_01.atlas.txt"),
    );
    let skeleton_handle = skeletons.add(skeleton);
    commands.spawn(SpineBundle {
        skeleton: skeleton_handle.clone().into(),
        transform: Transform::from_xyz(0., 0., -2500.).with_scale(Vec3::ONE),
        settings: SpineSettings {
            default_materials: false,
            mesh_type: SpineMeshType::Mesh3D,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn on_spawn(
    asset_server: Res<AssetServer>,
    mut spine_ready_event: EventReader<SpineReadyEvent>,
    mut spine_query: Query<&mut Spine>,
    mut commands: Commands,
) {
    let mut animation_list = Vec::new();
    for event in spine_ready_event.read() {
        if let Ok(mut spine) = spine_query.get_mut(event.entity) {
            let Spine(SkeletonController {
                skeleton,
                animation_state,
                ..
            }) = spine.as_mut();
            let mask_slots: Vec<String> = skeleton
                .slots()
                .filter_map(|s| {
                    let slot_data = s.data();
                    let slot_name = slot_data.name();
                    if slot_name.starts_with("MaskMosaic") || slot_name.starts_with("Penis") {
                        Some(slot_name.to_string())
                    } else {
                        None
                    }
                }).collect();
            for slot_name in mask_slots {
                skeleton.set_attachment(&slot_name, None);
            }
            for i in animation_state.data().skeleton_data().animations() {
                animation_list.push(i.name().to_string());
            }
        }
    }

    commands.spawn((
        Visibility::Visible,
        Node {
            width: Val::Percent(11.),
            height: Val::Percent(96.),
            left: Val::Percent(1.),
            top: Val::Percent(1.),
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )).with_children(|parent| {
        for (i, animation) in animation_list.iter().enumerate() {
            if i == 0 {
                parent.spawn((
                    Text::new("Animation List"),
                    TextFont {
                        font: asset_server.load("FOT-KurokaneStd-EB.otf"),
                        font_size: 20.,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.8, 0.7)),
                ));
            }
            parent.spawn((
                Button,
                Node {
                    width: Val::Percent(90.),
                    ..default()
                },
                Text::new(animation),
                TextFont {
                    font: asset_server.load("FOT-KurokaneStd-EB.otf"),
                    font_size: 20.,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.8, 0.2)),
                BackgroundColor(Color::srgb(0., 0., 0.)),
            ));
        }
    });
}

fn choose_animation(
    mut interaction_query: Query<(
        &Interaction,
        &Text,
        &mut TextColor,
        &mut BackgroundColor,
    ), (Changed<Interaction>, With<Button>),>,
    mut spine_query: Query<&mut Spine>,
) {
    for (interaction, text, mut color, mut bg_color) in &mut interaction_query {
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
