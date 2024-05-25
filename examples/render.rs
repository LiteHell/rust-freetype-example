use clap::Parser;
use freetype_example::font::Font;

/// Rendering example
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Font file name
    #[arg(short, long)]
    font: String,

    /// Text
    #[arg(short, long)]
    text: String,

    /// Output image path
    #[arg(short, long)]
    output: String,

    /// Vertical dpi
    #[arg(short, long, default_value_t = 300)]
    vdpi: u32,

    /// Horizontal dpi
    #[arg(short, long, default_value_t = 300)]
    hdpi: u32,

    /// Font size, in pt unit
    #[arg(short, long, default_value_t = 20.0)]
    font_size: f32,
}

pub fn main() {
    let args = Args::parse();
    let text = args.text.as_str();
    let mut face = Font::from_file(args.font.as_str(), 0);
    face.set_dpi(args.hdpi, args.vdpi);
    face.set_font_size(args.font_size);

    let size = face.measure_size(text).unwrap();
    let result = face.render(text).unwrap();

    let mut imgbuf = image::ImageBuffer::new(size.width as u32, size.height as u32);

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let rgba = result.get_rgba(x as i64, y as i64);
        *pixel = image::Rgba([rgba.0, rgba.1, rgba.2, rgba.3]);
    }

    imgbuf
        .save(args.output.as_str())
        .expect("Failed to save image");
}
