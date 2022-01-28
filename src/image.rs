use std::num::NonZeroU32;

use crate::image_view::{ImageRows, ImageRowsMut, TypedImageView, TypedImageViewMut};
use crate::pixels::{Pixel, PixelType, U16x3, U8x3, U8x4, F32, I32, U16, U8};
use crate::{ImageBufferError, ImageView, ImageViewMut, InvalidBufferSizeError};

#[derive(Debug)]
enum PixelsContainer<'a> {
    MutU32(&'a mut [u32]),
    MutU16(&'a mut [u16]),
    MutU8(&'a mut [u8]),
    VecU32(Vec<u32>),
    VecU8(Vec<u8>),
    VecU16(Vec<u16>),
}

/// Simple image container.
#[derive(Debug)]
pub struct Image<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: PixelsContainer<'a>,
    pixel_type: PixelType,
}

impl<'a> Image<'a> {
    /// Create empty image with given dimensions and pixel type.
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixel_type: PixelType) -> Self {
        let pixels_count = (width.get() * height.get()) as usize;
        let pixels = match pixel_type {
            PixelType::U8x3 => PixelsContainer::VecU8(vec![0; pixels_count * U8x3::size()]),
            PixelType::U16x3 => PixelsContainer::VecU8(vec![0; pixels_count * U16x3::size()]),
            PixelType::U8x4 | PixelType::I32 | PixelType::F32 => {
                PixelsContainer::VecU32(vec![0; pixels_count])
            }
            PixelType::U8 => PixelsContainer::VecU8(vec![0; pixels_count]),
            PixelType::U16 => PixelsContainer::VecU16(vec![0; pixels_count]),
        };
        Self {
            width,
            height,
            pixels,
            pixel_type,
        }
    }

    pub fn from_vec_u32(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: Vec<u32>,
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if buffer.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::VecU32(buffer),
            pixel_type,
        })
    }

    pub fn from_vec_u8(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: Vec<u8>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(&buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::VecU8(buffer),
            pixel_type,
        })
    }

    pub fn from_slice_u32(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u32],
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if buffer.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::MutU32(buffer),
            pixel_type,
        })
    }

    pub fn from_slice_u16(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u16],
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if buffer.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::MutU16(buffer),
            pixel_type,
        })
    }

    pub fn from_slice_u8(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::MutU8(buffer),
            pixel_type,
        })
    }

    #[inline(always)]
    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    /// Buffer with image pixels.
    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        match &self.pixels {
            PixelsContainer::MutU32(p) => unsafe { p.align_to::<u8>().1 },
            PixelsContainer::MutU16(p) => unsafe { p.align_to::<u8>().1 },
            PixelsContainer::MutU8(p) => *p,
            PixelsContainer::VecU32(v) => unsafe { v.align_to::<u8>().1 },
            PixelsContainer::VecU8(v) => v,
            PixelsContainer::VecU16(v) => unsafe { v.align_to::<u8>().1 },
        }
    }

    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut [u8] {
        match &mut self.pixels {
            PixelsContainer::MutU32(p) => unsafe { p.align_to_mut::<u8>().1 },
            PixelsContainer::MutU16(p) => unsafe { p.align_to_mut::<u8>().1 },
            PixelsContainer::MutU8(p) => p,
            PixelsContainer::VecU32(ref mut v) => unsafe { v.align_to_mut::<u8>().1 },
            PixelsContainer::VecU16(ref mut v) => unsafe { v.align_to_mut::<u8>().1 },
            PixelsContainer::VecU8(ref mut v) => v.as_mut_slice(),
        }
    }

    #[inline(always)]
    pub fn view(&self) -> ImageView {
        let buffer = self.buffer();
        let rows = match self.pixel_type {
            PixelType::U8x3 => {
                let pixels = unsafe { buffer.align_to::<U8x3>().1 };
                ImageRows::U8x3(pixels.chunks_exact(self.width.get() as usize).collect())
            }
            PixelType::U8x4 => {
                let pixels = unsafe { buffer.align_to::<U8x4>().1 };
                ImageRows::U8x4(pixels.chunks_exact(self.width.get() as usize).collect())
            }
            PixelType::U16x3 => {
                let pixels = unsafe { buffer.align_to::<U16x3>().1 };
                ImageRows::U16x3(pixels.chunks_exact(self.width.get() as usize).collect())
            }
            PixelType::I32 => {
                let pixels = unsafe { buffer.align_to::<I32>().1 };
                ImageRows::I32(pixels.chunks_exact(self.width.get() as usize).collect())
            }
            PixelType::F32 => {
                let pixels = unsafe { buffer.align_to::<F32>().1 };
                ImageRows::F32(pixels.chunks_exact(self.width.get() as usize).collect())
            }
            PixelType::U8 => {
                let pixels = unsafe { buffer.align_to::<U8>().1 };
                ImageRows::U8(pixels.chunks_exact(self.width.get() as usize).collect())
            }
            PixelType::U16 => {
                let pixels = unsafe { buffer.align_to::<u16>().1 };
                ImageRows::U16(pixels.chunks_exact(self.width.get() as usize).collect())
            }
        };
        ImageView::new(self.width, self.height, rows).unwrap()
    }

    #[inline(always)]
    pub fn view_mut(&mut self) -> ImageViewMut {
        let pixel_type = self.pixel_type;
        let width = self.width;
        let height = self.height;
        let buffer = self.buffer_mut();
        let rows = match pixel_type {
            PixelType::U8x3 => {
                let pixels = unsafe { buffer.align_to_mut::<U8x3>().1 };
                ImageRowsMut::U8x3(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
            PixelType::U8x4 => {
                let pixels = unsafe { buffer.align_to_mut::<U8x4>().1 };
                ImageRowsMut::U8x4(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
            PixelType::U16x3 => {
                let pixels = unsafe { buffer.align_to_mut::<U16x3>().1 };
                ImageRowsMut::U16x3(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
            PixelType::I32 => {
                let pixels = unsafe { buffer.align_to_mut::<I32>().1 };
                ImageRowsMut::I32(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
            PixelType::F32 => {
                let pixels = unsafe { buffer.align_to_mut::<F32>().1 };
                ImageRowsMut::F32(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
            PixelType::U8 => {
                let pixels = unsafe { buffer.align_to_mut::<U8>().1 };
                ImageRowsMut::U8(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
            PixelType::U16 => {
                let pixels = unsafe { buffer.align_to_mut::<u16>().1 };
                ImageRowsMut::U16(pixels.chunks_exact_mut(width.get() as usize).collect())
            }
        };
        ImageViewMut::new(width, height, rows).unwrap()
    }
}

/// Generic image container for internal purposes.
pub(crate) struct InnerImage<'a, P>
where
    P: Pixel,
{
    width: NonZeroU32,
    height: NonZeroU32,
    rows: Vec<&'a mut [P]>,
}

impl<'a, P> InnerImage<'a, P>
where
    P: Pixel,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixels: &'a mut [P]) -> Self {
        let rows = pixels.chunks_mut(width.get() as usize).collect();
        Self {
            width,
            height,
            rows,
        }
    }

    #[inline(always)]
    pub fn src_view<'s>(&'s self) -> TypedImageView<'s, 'a, P> {
        let rows = self.rows.as_slice();
        let rows: &[&[P]] = unsafe { std::mem::transmute(rows) };
        TypedImageView::new(self.width, self.height, rows)
    }

    #[inline(always)]
    pub fn dst_view<'s>(&'s mut self) -> TypedImageViewMut<'s, 'a, P> {
        TypedImageViewMut::new(self.width, self.height, self.rows.as_mut_slice())
    }
}
