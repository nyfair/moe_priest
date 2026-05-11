use bevy::prelude::*;
use bevy_tweening::{RepeatCount, RepeatStrategy};
use std::str::FromStr;
use std::time::Duration;

use crate::utage4::Node;

#[derive(Debug)]
pub struct Tween {
    pub target: String,
    pub tween_type: TweenType,
    pub params: TweenParams,
    pub ease_type: EaseFunction,
    pub loop_type: RepeatStrategy,
    pub loop_count: RepeatCount,
}

macro_rules! tween_type {
    ($($name:ident),* $(,)?) => {
        #[derive(Debug, PartialEq)]
        pub enum TweenType {
            $($name,)*
        }

        impl TweenType {
            fn parse(s: &str) -> Option<Self> {
                match s {
                    $(stringify!($name) => Some(TweenType::$name),)*
                    _ => None,
                }
            }
        }
    };
}

tween_type![
    MoveBy, MoveTo, MoveFrom, MoveAdd,
    PunchPosition, ShakePosition,
    RotateTo, RotateFrom, RotateBy, RotateAdd,
    PunchRotation, ShakeRotation,
    ScaleTo, ScaleFrom, ScaleBy, ScaleAdd,
    PunchScale, ShakeScale,
    ColorTo, ColorFrom,
];

#[derive(Debug, Default)]
pub struct TweenParams {
    pub time: Duration,
    pub delay: Duration,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub islocal: Option<bool>,
    pub color: Option<String>,
    pub alpha: Option<f32>,
    pub speed: Option<f32>,
    pub r: Option<f32>,
    pub g: Option<f32>,
    pub b: Option<f32>,
    pub a: Option<f32>,
}

impl TweenParams {
    fn parse(s: &str) -> Self {
        let mut params = TweenParams::default();
        for part in s.split_whitespace() {
            let mut kv = part.splitn(2, '=');
            if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
                match key {
                    "time" => params.time = Duration::from_secs_f32(
                            f32::from_str(value).unwrap_or(1.)).max(Duration::from_nanos(1)),
                    "delay" => params.delay = Duration::from_secs_f32(
                            f32::from_str(value).unwrap_or(0.)).max(Duration::from_nanos(1)),
                    "x" => params.x = f32::from_str(value).ok(),
                    "y" => params.y = f32::from_str(value).ok(),
                    "z" => params.z = f32::from_str(value).ok(),
                    "islocal" => params.islocal = bool::from_str(value).ok(),
                    "color" => params.color = Some(value.to_string()),
                    "alpha" => params.alpha = f32::from_str(value).ok(),
                    "speed" => params.speed = f32::from_str(value).ok(),
                    "r" => params.r = f32::from_str(value).ok(),
                    "g" => params.g = f32::from_str(value).ok(),
                    "b" => params.b = f32::from_str(value).ok(),
                    "a" => params.a = f32::from_str(value).ok(),
                    _ => {}
                }
            }
        }
        params
    }
}

fn parse_ease_type(s: &str) -> EaseFunction {
    match s {
        "easeInQuad" => EaseFunction::QuadraticIn,
        "easeOutQuad" => EaseFunction::QuadraticOut,
        "easeInOutQuad" => EaseFunction::QuadraticInOut,
        "easeInCubic" => EaseFunction::CubicIn,
        "easeOutCubic" => EaseFunction::CubicOut,
        "easeInOutCubic" => EaseFunction::CubicInOut,
        "easeInQuart" => EaseFunction::QuarticIn,
        "easeOutQuart" => EaseFunction::QuadraticOut,
        "easeInOutQuart" => EaseFunction::QuadraticInOut,
        "easeInQuint" => EaseFunction::QuinticIn,
        "easeOutQuint" => EaseFunction::QuinticOut,
        "easeInOutQuint" => EaseFunction::QuinticInOut,
        "easeInSine" => EaseFunction::SineIn,
        "easeOutSine" => EaseFunction::SineOut,
        "easeInOutSine" => EaseFunction::SineInOut,
        "easeInExpo" => EaseFunction::ExponentialIn,
        "easeOutExpo" => EaseFunction::ExponentialOut,
        "easeInOutExpo" => EaseFunction::ExponentialInOut,
        "easeInCirc" => EaseFunction::CircularIn,
        "easeOutCirc" => EaseFunction::CircularOut,
        "easeInOutCirc" => EaseFunction::CircularInOut,
        "easeInBounce" => EaseFunction::BounceIn,
        "easeOutBounce" => EaseFunction::BounceOut,
        "easeInOutBounce" => EaseFunction::BounceInOut,
        "easeInBack" => EaseFunction::BackIn,
        "easeOutBack" => EaseFunction::BackOut,
        "easeInOutBack" => EaseFunction::BackInOut,
        "easeInElastic" => EaseFunction::ElasticIn,
        "easeOutElastic" => EaseFunction::ElasticOut,
        "easeInOutElastic" => EaseFunction::ElasticInOut,
        _ => EaseFunction::Linear,
    }
}

fn parse_loop_type(s: &str) -> (RepeatStrategy, RepeatCount) {
    let (strategy, prefix) = if s.starts_with("loop") {
        (RepeatStrategy::Repeat, "loop=")
    } else if s.starts_with("pingPong") {
        (RepeatStrategy::MirroredRepeat, "pingPong=")
    } else {
        return (RepeatStrategy::Repeat, RepeatCount::Finite(1));
    };
    let count = s.strip_prefix(prefix)
        .and_then(|n| n.parse().ok())
        .map(|i| if i == 0 { RepeatCount::Infinite } else { RepeatCount::Finite(i) })
        .unwrap_or(RepeatCount::Infinite);
    (strategy, count)
}

impl Tween {
    pub fn new(node: &Node) -> Option<Self> {
        if node.command.as_deref() != Some("Tween") {
            return None;
        }
        let target = node.arg1.clone()?;
        let tween_type = node.arg2.as_deref().and_then(TweenType::parse)?;
        let params = node.arg3.as_deref().map(TweenParams::parse)?;
        let ease_type = node.arg4.as_deref().map(parse_ease_type).unwrap_or(EaseFunction::Linear);
        let (loop_type, loop_count) = node.arg5.as_deref().map(parse_loop_type)
            .unwrap_or((RepeatStrategy::Repeat, RepeatCount::Finite(1)));
        Some(Self {
            target,
            tween_type,
            params,
            ease_type,
            loop_type,
            loop_count,
        })
    }
}
