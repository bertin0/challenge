#![allow(rustdoc::missing_crate_level_docs)]

use std::{io::BufRead, time::Instant};

use egui::ColorImage;
use image::{
    Rgb,
    imageops::{FilterType, resize},
};
use imageproc::{
    geometric_transformations::{Interpolation, Projection, warp},
    map::map_colors,
};
use nokhwa::{
    Camera, NokhwaError,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::new(MyApp::new(cc)))),
    )
}

struct MyApp {
    camera: Camera,
    texture: egui::TextureHandle,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        MyApp {
            camera: setup_camera().expect("Unable to open camera."),
            texture: cc.egui_ctx.load_texture(
                "camera",
                ColorImage::example(),
                egui::TextureOptions::default(),
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = self.camera.frame().expect("Cannot get frame");

        let decoded = frame
            .decode_image::<RgbFormat>()
            .expect("Cannot decode frame");

        let now = Instant::now();

        let decoded = map_colors(&decoded, |p| Rgb([255 - p[0], 255 - p[1], 255 - p[2]]));
        let decoded = warp(
            &decoded,
            &(Projection::from_matrix([1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0]).unwrap()
                * Projection::translate(0.0, decoded.height() as f32 * -1.0)),
            Interpolation::Bilinear,
            Rgb([0, 0, 0]),
        );
        let decoded = resize(
            &decoded,
            decoded.width() / 2,
            decoded.height() / 2,
            FilterType::Nearest,
        );
        println!("Effects time: {:#?}", now.elapsed());

        let image = egui::ColorImage::from_rgb(
            [decoded.width() as usize, decoded.height() as usize],
            &decoded.as_raw(),
        );

        self.texture.set(image, egui::TextureOptions::default());

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.image(&self.texture);
        });

        ctx.request_repaint();
    }
}

fn setup_camera() -> Result<Camera, NokhwaError> {
    let index = CameraIndex::Index(0);

    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    let mut camera = Camera::new(index, requested)?;

    camera.open_stream()?;

    return Ok(camera);
}
