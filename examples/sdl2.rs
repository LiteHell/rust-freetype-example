use clap::Parser;
use rust_freetype_harfbuzz_example::{bitmap::StringBitmap, font::Font, string_bitmap_to_texture};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

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

    /// Window width
    #[arg(long, default_value_t = 800)]
    window_width: u32,

    /// Window height
    #[arg(long, default_value_t = 800)]
    window_height: u32,
}

pub fn render(args: &Args) -> StringBitmap {
    let text = args.text.as_str();
    let mut face = Font::from_file(args.font.as_str(), 0);
    face.set_dpi(args.hdpi, args.vdpi);
    face.set_font_size(args.font_size);
    face.set_letter_spacing(args.letter_spacing);

    face.render(text).unwrap()
}

pub fn main() -> Result<(), String> {
    let args = Args::parse();
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Freetyper sdl2 example",
            args.window_width,
            args.window_height,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let texture_creator = canvas.texture_creator();
    let texture = {
        let bitmap = render(&args);
        string_bitmap_to_texture!(bitmap, texture_creator)
    }
    .expect("Failed to create texture");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.clear();
        canvas
            .copy(
                &texture,
                None,
                Rect::new(0, 0, texture.query().width, texture.query().height),
            )
            .unwrap();
        canvas.present();
    }

    Ok(())
}
