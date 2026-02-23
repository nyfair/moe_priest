use std::str::FromStr;
use crate::utage4::Node;

#[derive(Debug)]
pub struct Tween {
    pub target: String,
    pub tween_type: TweenType,
    pub params: TweenParams,
    pub ease_type: EaseType,
    pub loop_type: Option<LoopType>,
}

#[derive(Debug, PartialEq)]
pub enum TweenType {
    MoveBy,
    // MoveTo, MoveFrom, MoveAdd,
    PunchPosition,
    ShakePosition,
    // RotateTo, RotateFrom, RotateBy, RotateAdd, PunchRotation, ShakeRotation,
    // ScaleTo, ScaleFrom, ScaleBy, ScaleAdd, PunchScale, ShakeScale,
    ColorTo,
    // ColorFrom,
}

#[derive(Debug, Default)]
pub struct TweenParams {
    pub time: Option<f32>,
    pub speed: Option<f32>,
    pub delay: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub islocal: Option<bool>,
    pub color: Option<String>,
    pub alpha: Option<f32>,
    pub r: Option<f32>,
    pub g: Option<f32>,
    pub b: Option<f32>,
    pub a: Option<f32>,
}

#[derive(Debug, Default, PartialEq)]
pub enum EaseType {
    #[default]
    Linear,
    EaseInQuart,
}

#[derive(Debug, PartialEq)]
pub enum LoopType {
    Loop(Option<u32>),
    PingPong(Option<u32>),
}


fn parse_tween_type(s: &str) -> Option<TweenType> {
    match s {
        // "MoveTo" => Some(TweenType::MoveTo),
        // "MoveFrom" => Some(TweenType::MoveFrom),
        "MoveBy" => Some(TweenType::MoveBy),
        // "MoveAdd" => Some(TweenType::MoveAdd),
        "PunchPosition" => Some(TweenType::PunchPosition),
        "ShakePosition" => Some(TweenType::ShakePosition),
        // "RotateTo" => Some(TweenType::RotateTo),
        // "RotateFrom" => Some(TweenType::RotateFrom),
        // "RotateBy" => Some(TweenType::RotateBy),
        // "RotateAdd" => Some(TweenType::RotateAdd),
        // "PunchRotation" => Some(TweenType::PunchRotation),
        // "ShakeRotation" => Some(TweenType::ShakeRotation),
        // "ScaleTo" => Some(TweenType::ScaleTo),
        // "ScaleFrom" => Some(TweenType::ScaleFrom),
        // "ScaleBy" => Some(TweenType::ScaleBy),
        // "ScaleAdd" => Some(TweenType::ScaleAdd),
        // "PunchScale" => Some(TweenType::PunchScale),
        // "ShakeScale" => Some(TweenType::ShakeScale),
        "ColorTo" => Some(TweenType::ColorTo),
        // "ColorFrom" => Some(TweenType::ColorFrom),
        _ => None,
    }
}

fn parse_tween_params(s: &str) -> TweenParams {
    let mut params = TweenParams::default();
    for part in s.split_whitespace() {
        let mut kv = part.splitn(2, '=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            match key {
                "time" => params.time = f32::from_str(value).ok(),
                "speed" => params.speed = f32::from_str(value).ok(),
                "delay" => params.delay = f32::from_str(value).ok(),
                "x" => params.x = f32::from_str(value).ok(),
                "y" => params.y = f32::from_str(value).ok(),
                "z" => params.z = f32::from_str(value).ok(),
                "islocal" => params.islocal = bool::from_str(value).ok(),
                "color" => params.color = Some(value.to_string()),
                "alpha" => params.alpha = f32::from_str(value).ok(),
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

fn parse_ease_type(s: &str) -> EaseType {
    match s {
        "easeInQuart" => EaseType::EaseInQuart,
        // "spring" => EaseType::Spring,
        // "easeInQuad" => EaseType::EaseInQuad,
        // "easeOutQuad" => EaseType::EaseOutQuad,
        // "easeInOutQuad" => EaseType::EaseInOutQuad,
        // "easeInCubic" => EaseType::EaseInCubic,
        // "easeOutCubic" => EaseType::EaseOutCubic,
        // "easeInOutCubic" => EaseType::EaseInOutCubic,
        "linear" | _ => EaseType::Linear,
    }
}

fn parse_loop_type(s: &str) -> Option<LoopType> {
    if s.starts_with("loop") {
        let count = s.strip_prefix("loop=").and_then(|num| u32::from_str(num).ok());
        Some(LoopType::Loop(count))
    } else if s.starts_with("pingPong") {
        let count = s.strip_prefix("pingPong=").and_then(|num| u32::from_str(num).ok());
        Some(LoopType::PingPong(count))
    } else {
        None
    }
}

impl Tween {
    pub fn new(node: &Node) -> Option<Self> {
        if node.command.as_deref() != Some("Tween") {
            return None;
        }
        let target = node.arg1.clone()?;
        let tween_type = parse_tween_type(node.arg2.as_deref()?)?;
        let params = node.arg3.as_deref().map(parse_tween_params).unwrap_or_default();
        let ease_type = node.arg4.as_deref().map(parse_ease_type).unwrap_or_default();
        let loop_type = node.arg5.as_deref().and_then(parse_loop_type);
        Some(Self {
            target,
            tween_type,
            params,
            ease_type,
            loop_type,
        })
    }
}
