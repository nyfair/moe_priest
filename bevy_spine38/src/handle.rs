use crate::{Crossfades, SkeletonData, SpineLoader, SpineSettings};
use bevy::prelude::*;

#[derive(Default, Component)]
#[require(SpineLoader, SpineSettings, Crossfades, Transform, Visibility)]
pub struct SkeletonDataHandle(pub Handle<SkeletonData>);

impl From<Handle<SkeletonData>> for SkeletonDataHandle {
    fn from(handle: Handle<SkeletonData>) -> Self {
        Self(handle)
    }
}
