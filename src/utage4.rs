use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InputRow {
    strings: Vec<String>,
    is_comment_out: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct InputSettingItem {
    rows: Vec<InputRow>,
    name: String,
    header_row: usize,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Root {
    setting_list: Vec<InputSettingItem>,
}

#[derive(Debug, Default, Serialize)]
pub struct VNConfig {
    pub character: Vec<CharacterEntry>,
    pub layer: Vec<LayerEntry>,
    pub param: Vec<ParamEntry>,
    pub sound: Vec<SoundEntry>,
    pub texture: Vec<TextureEntry>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterEntry {
    pub label: Option<String>,
    pub character_name: Option<String>,
    pub name_text: Option<String>,
    pub pattern: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub z: Option<String>,
    pub pivot: Option<String>,
    pub scale: Option<String>,
    pub conditional: Option<String>,
    pub file_name: Option<String>,
    pub sub_file_name: Option<String>,
    pub file_type: Option<String>,
    pub animation: Option<String>,
    pub render_texture: Option<String>,
    pub render_rect: Option<String>,
    pub eye_blink: Option<String>,
    pub lip_synch: Option<String>,
    pub icon: Option<String>,
    pub icon_sub_file_name: Option<String>,
    pub icon_rect: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerEntry {
    pub layer_name: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub order: Option<String>,
    pub layer_mask: Option<String>,
    pub scale_x: Option<String>,
    pub scale_y: Option<String>,
    pub flip_x: Option<String>,
    pub flip_y: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub border_left: Option<String>,
    pub border_right: Option<String>,
    pub border_top: Option<String>,
    pub border_bottom: Option<String>,
    pub align: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamEntry {
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub value: Option<String>,
    pub file_type: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundEntry {
    pub label: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub file_name: Option<String>,
    pub intro_time: Option<String>,
    pub volume: Option<String>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEntry {
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub z: Option<String>,
    pub pivot: Option<String>,
    pub scale: Option<String>,
    pub conditional: Option<String>,
    pub sub_file_name: Option<String>,
    pub animation: Option<String>,
    pub render_texture: Option<String>,
    pub render_rect: Option<String>,
    pub thumbnail: Option<String>,
    pub cg_categolly: Option<String>,
}

fn row_to_map<'a>(row: &'a InputRow, headers: &'a [String]) -> HashMap<&'a str, &'a str> {
    headers
        .iter()
        .enumerate()
        .filter_map(|(i, key)| {
            if !key.is_empty() {
                row.strings.get(i).map(|value| (key.as_str(), value.as_str()))
            } else {
                None
            }
        })
        .collect()
}

pub fn parse_chapter() -> Result<VNConfig, Box<dyn Error>> {
    let json_content = fs::read_to_string("assets/advscene/scenariochapter/config.chapter.json")?;
    let root: Root = serde_json::from_str(&json_content)?;
    let mut cfg = VNConfig::default();

    for setting in root.setting_list {
        let headers = if let Some(header_row) = setting.rows.get(setting.header_row) {
            &header_row.strings
        } else {
            continue;
        };

        match setting.name.as_str() {
            s if s.contains("cfg_character.xlsx:Character") => {
                for row in &setting.rows {
                    if row.is_comment_out == 1 { continue; }
                    let map = row_to_map(row, headers);
                    if map.is_empty() { continue; }

                    let entry = CharacterEntry {
                        label: Some(row.strings[2].clone()),
                        character_name: map.get("CharacterName").map(|s| s.to_string()),
                        name_text: map.get("NameText").map(|s| s.to_string()),
                        pattern: map.get("Pattern").map(|s| s.to_string()),
                        x: map.get("X").map(|s| s.to_string()),
                        y: map.get("Y").map(|s| s.to_string()),
                        z: map.get("Z").map(|s| s.to_string()),
                        pivot: map.get("Pivot").map(|s| s.to_string()),
                        scale: map.get("Scale").map(|s| s.to_string()),
                        conditional: map.get("Conditional").map(|s| s.to_string()),
                        file_name: map.get("FileName").map(|s| s.to_string()),
                        sub_file_name: map.get("SubFileName").map(|s| s.to_string()),
                        file_type: map.get("FileType").map(|s| s.to_string()),
                        animation: map.get("Animation").map(|s| s.to_string()),
                        render_texture: map.get("RenderTexture").map(|s| s.to_string()),
                        render_rect: map.get("RenderRect").map(|s| s.to_string()),
                        eye_blink: map.get("EyeBlink").map(|s| s.to_string()),
                        lip_synch: map.get("LipSynch").map(|s| s.to_string()),
                        icon: map.get("Icon").map(|s| s.to_string()),
                        icon_sub_file_name: map.get("IconSubFileName").map(|s| s.to_string()),
                        icon_rect: map.get("IconRect").map(|s| s.to_string()),
                    };
                    cfg.character.push(entry);
                }
            }

            s if s.contains("cfg_layer.xlsx:Layer") => {
                 for row in &setting.rows {
                    if row.is_comment_out == 1 { continue; }
                    let map = row_to_map(row, headers);
                    if map.is_empty() { continue; }
                    cfg.layer.push(LayerEntry {
                        layer_name: map.get("LayerName").map(|s| s.to_string()),
                        entry_type: map.get("Type").map(|s| s.to_string()),
                        x: map.get("X").map(|s| s.to_string()),
                        y: map.get("Y").map(|s| s.to_string()),
                        order: map.get("Order").map(|s| s.to_string()),
                        layer_mask: map.get("LayerMask").map(|s| s.to_string()),
                        scale_x: map.get("ScaleX").map(|s| s.to_string()),
                        scale_y: map.get("ScaleY").map(|s| s.to_string()),
                        flip_x: map.get("FlipX").map(|s| s.to_string()),
                        flip_y: map.get("FlipY").map(|s| s.to_string()),
                        width: map.get("Width").map(|s| s.to_string()),
                        height: map.get("Height").map(|s| s.to_string()),
                        border_left: map.get("BorderLeft").map(|s| s.to_string()),
                        border_right: map.get("BorderRight").map(|s| s.to_string()),
                        border_top: map.get("BorderTop").map(|s| s.to_string()),
                        border_bottom: map.get("BorderBottom").map(|s| s.to_string()),
                        align: map.get("Align").map(|s| s.to_string()),
                    });
                }
            }

            s if s.contains("cfg_param.xlsx:Param") => {
                for row in &setting.rows {
                    if row.is_comment_out == 1 { continue; }
                    let map = row_to_map(row, headers);
                    if map.is_empty() { continue; }
                    cfg.param.push(ParamEntry {
                        label: map.get("Label").map(|s| s.to_string()),
                        entry_type: map.get("Type").map(|s| s.to_string()),
                        value: map.get("Value").map(|s| s.to_string()),
                        file_type: map.get("FileType").map(|s| s.to_string()),
                    });
                }
            }

            s if s.contains("cfg_sound.xlsx:Sound") => {
                for row in &setting.rows {
                    if row.is_comment_out == 1 { continue; }
                    let map = row_to_map(row, headers);
                    if map.is_empty() { continue; }
                    cfg.sound.push(SoundEntry {
                        label: map.get("Label").map(|s| s.to_string()),
                        title: map.get("Title").map(|s| s.to_string()),
                        entry_type: map.get("Type").map(|s| s.to_string()),
                        file_name: map.get("FileName").map(|s| s.to_string()),
                        intro_time: map.get("IntroTime").map(|s| s.to_string()),
                        volume: map.get("Volume").map(|s| s.to_string()),
                    });
                }
            }

            s if s.contains("cfg_texture.xlsx:Texture") => {
                for row in &setting.rows {
                    if row.is_comment_out == 1 { continue; }
                    let map = row_to_map(row, headers);
                    if map.is_empty() { continue; }
                    cfg.texture.push(TextureEntry {
                        label: map.get("Label").map(|s| s.to_string()),
                        entry_type: map.get("Type").map(|s| s.to_string()),
                        file_name: map.get("FileName").map(|s| s.to_string()),
                        file_type: map.get("FileType").map(|s| s.to_string()),
                        x: map.get("X").map(|s| s.to_string()),
                        y: map.get("Y").map(|s| s.to_string()),
                        z: map.get("Z").map(|s| s.to_string()),
                        pivot: map.get("Pivot").map(|s| s.to_string()),
                        scale: map.get("Scale").map(|s| s.to_string()),
                        conditional: map.get("Conditional").map(|s| s.to_string()),
                        sub_file_name: map.get("SubFileName").map(|s| s.to_string()),
                        animation: map.get("Animation").map(|s| s.to_string()),
                        render_texture: map.get("RenderTexture").map(|s| s.to_string()),
                        render_rect: map.get("RenderRect").map(|s| s.to_string()),
                        thumbnail: map.get("Thumbnail").map(|s| s.to_string()),
                        cg_categolly: map.get("CgCategolly").map(|s| s.to_string()),
                    });
                }
            }
            _ => {}
        }
    }
    return Ok(cfg);
}
