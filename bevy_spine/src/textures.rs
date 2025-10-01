//! Events related to textures loaded by Spine.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use rusty_spine::atlas::{AtlasFilter, AtlasWrap};

#[derive(Debug)]
pub struct SpineTexture(pub String);

#[derive(Debug)]
struct SpineTextureInternal {
    pub path: String,
    pub config: SpineTextureConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct SpineTextureConfig {
    pub premultiplied_alpha: bool,
    pub min_filter: AtlasFilter,
    pub mag_filter: AtlasFilter,
    pub u_wrap: AtlasWrap,
    pub v_wrap: AtlasWrap,
}

#[derive(Resource)]
pub(crate) struct SpineTextures {
    data: Arc<Mutex<SpineTexturesData>>,
}

/// An [`Message`] fired for each texture loaded by Spine.
///
/// Sent in [`SpineSystem::Load`](`crate::SpineSystem::Load`).
#[derive(Debug, Clone, Message)]
pub struct SpineTextureCreateMsg {
    pub path: String,
    pub handle: Handle<Image>,
    pub config: SpineTextureConfig,
}

/// An [`Message`] fired for each texture disposed, after [`SpineTextureCreateEvent`].
///
/// Sent in [`SpineSystem::Load`](`crate::SpineSystem::Load`).
#[derive(Debug, Clone, Message)]
pub struct SpineTextureDisposeMsg {
    pub path: String,
    pub handle: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct SpineTexturesData {
    handles: Vec<(String, Handle<Image>)>,
    remember: Vec<SpineTextureInternal>,
    forget: Vec<String>,
}

impl SpineTextures {
    pub(crate) fn init() -> Self {
        let data = Arc::new(Mutex::new(SpineTexturesData::default()));

        let data2 = data.clone();
        rusty_spine::extension::set_create_texture_cb(move |page, path| {
            data2.lock().unwrap().remember.push(SpineTextureInternal {
                path: path.to_owned(),
                config: SpineTextureConfig {
                    //premultiplied_alpha: page.pma(),
                    premultiplied_alpha: true, // TODO
                    min_filter: page.min_filter(),
                    mag_filter: page.mag_filter(),
                    u_wrap: page.u_wrap(),
                    v_wrap: page.v_wrap(),
                },
            });
            page.renderer_object().set(SpineTexture(path.to_owned()));
        });

        let data3 = data.clone();
        rusty_spine::extension::set_dispose_texture_cb(move |page| unsafe {
            data3.lock().unwrap().forget.push(
                page.renderer_object()
                    .get_unchecked::<SpineTexture>()
                    .0
                    .clone(),
            );
            page.renderer_object().dispose::<SpineTexture>();
        });

        Self { data }
    }

    pub fn update(
        &self,
        asset_server: &AssetServer,
        create_events: &mut MessageWriter<SpineTextureCreateMsg>,
        dispose_events: &mut MessageWriter<SpineTextureDisposeMsg>,
    ) {
        let mut data = self.data.lock().unwrap();
        while let Some(texture) = data.remember.pop() {
            let handle = asset_server.load(&texture.path);
            data.handles.push((texture.path.clone(), handle.clone()));
            create_events.write(SpineTextureCreateMsg {
                path: texture.path,
                handle,
                config: texture.config,
            });
        }
        while let Some(texture_path) = data.forget.pop() {
            if let Some(index) = data.handles.iter().position(|i| i.0 == texture_path) {
                dispose_events.write(SpineTextureDisposeMsg {
                    path: texture_path,
                    handle: data.handles[index].1.clone(),
                });
                data.handles.remove(index);
            }
        }
    }
}
