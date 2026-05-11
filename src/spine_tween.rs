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
