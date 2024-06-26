use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use photon_rs::transform::SamplingFilter;
use prost::Message;

mod abi;

pub use abi::*;

impl ImageSpec {
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

impl From<&ImageSpec> for String {
    fn from(image_spec: &ImageSpec) -> Self {
        let data = image_spec.encode_to_vec();
        URL_SAFE_NO_PAD.encode(data)
    }
}

impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let data = URL_SAFE_NO_PAD.decode(value)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

impl From<resize::SampleFilter> for SamplingFilter {
    fn from(v: resize::SampleFilter) -> Self {
        match v {
            resize::SampleFilter::Unknown => SamplingFilter::Nearest,
            resize::SampleFilter::Nearest => SamplingFilter::Nearest,
            resize::SampleFilter::Triangle => SamplingFilter::Triangle,
            resize::SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            resize::SampleFilter::Gaussian => SamplingFilter::Gaussian,
            resize::SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

impl Spec {
    pub fn new_resize_seam_carve(width: u32, height: u32) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::SeamCarve as i32,
                filter: resize::SampleFilter::Unknown as i32,
            })),
        }
    }

    pub fn new_resize(width: u32, height: u32, filter: resize::SampleFilter) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::Normal as i32,
                filter: filter as i32,
            })),
        }
    }

    pub fn new_watermark(x: u32, y: u32) -> Self {
        Self {
            data: Some(spec::Data::Watermark(Watermark { x, y })),
        }
    }

    pub fn new_flipv() -> Self {
        Self {
            data: Some(spec::Data::Flipv(Flipv {})),
        }
    }
    pub fn new_fliph() -> Self {
        Self {
            data: Some(spec::Data::Fliph(Fliph {})),
        }
    }
    pub fn new_filter(f: filter::Filter) -> Self {
        let mut filter = f;
        if filter == filter::Filter::Unknown {
            filter = filter::Filter::Twenties;
        }
        Self {
            data: Some(spec::Data::Filter(Filter {
                filter: filter.into(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;

    #[test]
    fn encoded_spec_could_be_decoded() {
        let spec1 = Spec::new_resize(960, 720, resize::SampleFilter::CatmullRom);
        let spec2 = Spec::new_watermark(0, 0);
        let spec3 = Spec::new_fliph();
        let spec4 = Spec::new_filter(filter::Filter::Twenties);
        let image_spec = ImageSpec::new(vec![spec1, spec3, spec4, spec2]);
        let s: String = image_spec.borrow().into();
        println!("spec string: {}", s);
        assert_eq!(image_spec, s.as_str().try_into().unwrap());

        let image_spec2 = ImageSpec::try_from(s.as_str()).unwrap();
        assert_eq!(image_spec, image_spec2);
    }
}
