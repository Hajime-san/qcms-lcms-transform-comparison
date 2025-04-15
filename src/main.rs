fn main() {}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, ImageDecoder, codecs::png::PngDecoder};

    #[test]
    fn test_color_space_conversion() {
        // https://github.com/web-platform-tests/wpt/blob/d575dc75ede770df322fbc5da3112dcf81f192ec/images/wide-gamut-pattern.png
        let buf = std::fs::read("wide-gamut-pattern.png").unwrap();
        let mut img = PngDecoder::new(std::io::Cursor::new(&buf)).unwrap();
        let icc_profile = img.icc_profile().unwrap().unwrap();
        let img = DynamicImage::from_decoder(img).unwrap();
        let chunk_size = 8;
        let bytes = img.into_bytes();
        let mut lcms_chunks: Vec<u8> = Vec::new();
        let mut qcms_chunks: Vec<u8> = Vec::new();
        // lcms
        {
            let input_profile = lcms2::Profile::new_icc(&icc_profile).unwrap();
            let output_profile = lcms2::Profile::new_srgb();

            let transformer = lcms2::Transform::new(
                &input_profile,
                lcms2::PixelFormat::RGBA_16,
                &output_profile,
                lcms2::PixelFormat::RGBA_16,
                output_profile.header_rendering_intent(),
            )
            .unwrap();

            for pixel in bytes.chunks_exact(chunk_size) {
                let mut pixel = [
                    pixel[0], pixel[1], pixel[2], pixel[3], pixel[4], pixel[5], pixel[6], pixel[7],
                ];
                transformer.transform_in_place(&mut pixel);

                lcms_chunks.extend_from_slice(&pixel);
            }
        }
        // qcms
        {
            let input_profile = qcms::Profile::new_from_slice(&icc_profile, false).unwrap();
            let output_profile = qcms::Profile::new_sRGB();

            let transformer = qcms::Transform::new(
                &input_profile,
                &output_profile,
                qcms::DataType::RGBA8,
                qcms::Intent::default(),
            )
            .unwrap();

            for pixel in bytes.chunks_exact(chunk_size) {
                let mut pixel = [
                    pixel[0], pixel[1], pixel[2], pixel[3], pixel[4], pixel[5], pixel[6], pixel[7],
                ];
                transformer.apply(&mut pixel);

                qcms_chunks.extend_from_slice(&pixel);
            }
        }

        assert_eq!(lcms_chunks, qcms_chunks);
    }
}
