use image::DynamicImage;

#[allow(dead_code)]
pub fn crop_and_resize(
    img: &DynamicImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    new_w: u32,
    new_h: u32,
) -> DynamicImage {
    let cropped = img.crop_imm(x, y, w, h);
    cropped.resize(new_w, new_h, image::imageops::FilterType::Lanczos3)
}
