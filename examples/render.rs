use clap::Parser;
use freetyper::font::Font;

/// Rendering example
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Font file name
    #[arg(long)]
    font: String,

    /// Text
    #[arg(long)]
    text: String,

    /// Output image path
    #[arg(long)]
    output: String,

    /// Vertical dpi
    #[arg(long, default_value_t = 300)]
    vdpi: u32,

    /// Horizontal dpi
    #[arg(long, default_value_t = 300)]
    hdpi: u32,

    /// Font size, in pt unit
    #[arg(long, default_value_t = 20.0)]
    font_size: f32,

    /// Value to determine space between letters
    ///
    /// More bigger value, less space between letters
    #[arg(long, default_value_t = 1.2)]
    letter_spacing: f64,
}

pub fn main() {
    let args = Args::parse();
    let text = args.text.as_str();
    let mut face = Font::from_file(args.font.as_str(), 0);
    face.set_dpi(args.hdpi, args.vdpi);
    face.set_font_size(args.font_size);
    face.set_letter_spacing(args.letter_spacing);

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
