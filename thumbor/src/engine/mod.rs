use crate::pb::Spec;
use image::ImageOutputFormat;

mod photon;
pub use photon::Photon;

/// Engine represents image process engine
pub trait Engine {
    /// Process according to spec order
    fn apply(&mut self, specs: &[Spec]);
    /// generate target image with vec format
    fn generate(self, format: ImageOutputFormat) -> Vec<u8>;
}

/// SpecTransformer is used to transform
/// a image with given operator
pub trait SpecTransformer<T> {
    fn transform(&mut self, operator: T);
}
