pub struct SpineColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<Spine> for SpineColorLens {
    fn lerp(&mut self, mut target: Mut<Spine>, ratio: f32) {
        let rgba = self.start.mix(&self.end, ratio).to_linear();
        target.skeleton.set_color(rgba.red, rgba.green, rgba.blue, rgba.alpha);
    }
}

#[derive(PartialEq, Clone, Copy)]
enum ShakeKind {
    Punch,
    Shake,
}

#[derive(PartialEq, Clone, Copy)]
enum ShakeAxes {
    Position,
    Rotation,
    Scale,
}

#[derive(Component)]
struct ShakeAnim {
    kind: ShakeKind,
    axes: ShakeAxes,
    amp: Vec3,
    time: f32,
    duration: f32,
    delay: f32,
    base_pos: Vec3,
    base_rot: Quat,
    base_scale: Vec3,
    seed: u32,
}

fn shake_kind_axes(tween_type: &TweenType) -> (ShakeKind, ShakeAxes) {
    let kind = if matches!(tween_type,
        TweenType::PunchPosition | TweenType::PunchRotation | TweenType::PunchScale) {
        ShakeKind::Punch
    } else {
        ShakeKind::Shake
    };
    let axes = if matches!(tween_type,
        TweenType::PunchPosition | TweenType::ShakePosition) {
        ShakeAxes::Position
    } else if matches!(tween_type,
        TweenType::PunchRotation | TweenType::ShakeRotation) {
        ShakeAxes::Rotation
    } else {
        ShakeAxes::Scale
    };
    (kind, axes)
}

fn spawn_shake(
    commands: &mut Commands,
    entity: Entity,
    tween_type: &TweenType,
    amp: Vec3,
    duration: Duration,
    delay: Duration,
    base: (Vec3, Quat, Vec3),
    wait: bool,
) {
    let (kind, axes) = shake_kind_axes(tween_type);
    commands.entity(entity).insert(ShakeAnim {
        kind,
        axes,
        amp,
        time: 0.,
        duration: duration.as_secs_f32(),
        delay: delay.as_secs_f32(),
        base_pos: base.0,
        base_rot: base.1,
        base_scale: base.2,
        seed: entity.to_bits() as u32,
    });
    if wait {
        commands.entity(entity).insert(WaitEffect);
    }
}

fn rand_unit(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*seed >> 8) as f32 / 16777216.) * 2. - 1.
}

fn shake_anim(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ShakeAnim, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    query.iter_mut().for_each(|(entity, mut shake, mut transform)| {
        if shake.delay > 0. {
            shake.delay -= dt;
            return;
        }
        shake.time += dt;
        let t = shake.time / shake.duration.max(1e-6);
        if t >= 1. {
            transform.translation = shake.base_pos;
            transform.rotation = shake.base_rot;
            transform.scale = shake.base_scale;
            commands.entity(entity).remove::<ShakeAnim>();
            commands.entity(entity).remove::<WaitEffect>();
            return;
        }
        let off = match shake.kind {
            ShakeKind::Punch => {
                let k = (std::f32::consts::TAU * t).sin() * (1. - t);
                shake.amp * k
            }
            ShakeKind::Shake => {
                shake.amp * Vec3::new(
                    rand_unit(&mut shake.seed),
                    rand_unit(&mut shake.seed),
                    rand_unit(&mut shake.seed),
                ) * (1. - t)
            }
        };
        match shake.axes {
            ShakeAxes::Position => {
                transform.translation = shake.base_pos + off;
            }
            ShakeAxes::Rotation => {
                let rot = Quat::from_euler(
                    EulerRot::XYZ,
                    off.x.to_radians(),
                    off.y.to_radians(),
                    off.z.to_radians(),
                );
                transform.rotation = shake.base_rot * rot;
            }
            ShakeAxes::Scale => {
                transform.scale = shake.base_scale + off;
            }
        }
    });
}