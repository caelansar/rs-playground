use super::{Engine, SpecTransformer};
use crate::pb::*;
use anyhow::Result;
use bytes::Bytes;
use image::{DynamicImage, ImageBuffer, ImageOutputFormat};
use lazy_static::lazy_static;
use photon_rs::{multiple, native::open_image_from_bytes, transform, PhotonImage};
use std::io::Cursor;
use std::ops::{Deref, DerefMut};

lazy_static! {
    static ref WATERMARK: PhotonImage = {
        let data = include_bytes!("../../gopher.png");
        let watermark = open_image_from_bytes(data).unwrap();
        transform::resize(&watermark, 64, 64, transform::SamplingFilter::Nearest)
    };
}

pub struct Photon(PhotonImage);

impl Deref for Photon {
    type Target = PhotonImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Photon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<Bytes> for Photon {
    type Error = anyhow::Error;

    fn try_from(data: Bytes) -> Result<Self, Self::Error> {
        Ok(Self::new(open_image_from_bytes(&data)?))
    }
}

impl Photon {
    pub fn new(img: PhotonImage) -> Self {
        Self(img)
    }
}

impl Engine for Photon {
    fn apply(&mut self, specs: &[Spec]) {
        for spec in specs.iter() {
            match spec.data {
                Some(spec::Data::Resize(ref v)) => self.transform(v),
                Some(spec::Data::Watermark(ref v)) => self.transform(v),
                _ => unreachable!(),
            }
        }
    }

    fn generate(self, format: ImageOutputFormat) -> Vec<u8> {
        image_to_buf(&self, format)
    }
}

impl SpecTransformer<&Resize> for Photon {
    fn transform(&mut self, op: &Resize) {
        let img = match resize::ResizeType::from_i32(op.rtype).unwrap() {
            resize::ResizeType::Normal => transform::resize(
                &self,
                op.width,
                op.height,
                resize::SampleFilter::from_i32(op.filter).unwrap().into(),
            ),
            resize::ResizeType::SeamCarve => transform::seam_carve(&self, op.width, op.height),
        };
        self.0 = img;
    }
}

impl SpecTransformer<&Watermark> for Photon {
    fn transform(&mut self, op: &Watermark) {
        multiple::watermark(self, &WATERMARK, op.x, op.y);
    }
}

fn image_to_buf(img: &PhotonImage, format: ImageOutputFormat) -> Vec<u8> {
    let raw_pixels = img.get_raw_pixels();
    let width = img.get_width();
    let height = img.get_height();

    let img_buffer = ImageBuffer::from_vec(width, height, raw_pixels).unwrap();
    let dynimage = DynamicImage::ImageRgba8(img_buffer);

    let vec = Vec::with_capacity(32767);
    let mut buffer = Cursor::new(vec);

    dynimage.write_to(&mut buffer, format).unwrap();
    buffer.into_inner()
}
