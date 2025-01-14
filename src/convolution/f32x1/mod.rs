use super::{Coefficients, Convolution};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::F32;
use crate::CpuExtensions;

mod native;

impl Convolution for F32 {
    fn horiz_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            _ => native::horiz_convolution(src_image, dst_image, offset, coeffs),
        }
    }

    fn vert_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            _ => native::vert_convolution(src_image, dst_image, coeffs),
        }
    }
}
