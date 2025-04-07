use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use opencv::{
    core::*,
    highgui,
    imgproc::{INTER_AREA, resize},
    videoio::{self, CAP_PROP_FRAME_WIDTH, VideoCaptureTraitConst, *},
};

struct Effect {
    name: &'static str,
    // function: dyn Fn(&Mat, &mut Mat) -> (),
    function: fn(&Mat, &mut Mat) -> Result<(), opencv::Error>,
    enabled: Arc<AtomicBool>,
}

impl Effect {
    pub fn new(
        name: &'static str,
        function: fn(&Mat, &mut Mat) -> Result<(), opencv::Error>,
    ) -> Effect {
        let e = Effect {
            name,
            function,
            enabled: Arc::new(AtomicBool::new(false)),
        };

        let cloned = e.enabled.clone();

        let res = highgui::create_button(
            name,
            Some(Box::new(move |_| {
                cloned.fetch_xor(true, Ordering::Relaxed);
            })),
            highgui::QT_NEW_BUTTONBAR | highgui::QT_CHECKBOX,
            false,
        );

        match res {
            Ok(_) => {}
            Err(err) => eprintln!(
                "Cannot create button for effect {}: {}",
                e.name,
                err.to_string()
            ),
        }

        e
    }
}

pub fn camera_in() -> Result<(), opencv::Error> {
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_NORMAL)?;

    let invert = Effect::new("Invert", |src, mut dst| {
        bitwise_not(&src, &mut dst, &no_array())
    });

    let horiz_flip = Effect::new("Horizontal flip", |src, mut dst| flip(&src, &mut dst, 1));

    let vert_flip = Effect::new("Vertical flip", |src, mut dst| flip(&src, &mut dst, 0));

    let scale = Effect::new("Scale", |src, mut dst| {
        resize(&src, &mut dst, src.size()? * 2, 0.0, 0.0, INTER_AREA)
    });

    let effects = vec![invert, horiz_flip, vert_flip, scale];

    let mut cap = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    // cap.set(CAP_PROP_FRAME_WIDTH, 1280.0)?;
    // cap.set(CAP_PROP_FRAME_HEIGHT, 720.0)?;

    if !cap.is_opened()? {
        return Err(opencv::Error::new(1, "Failed to open camera"));
    }

    let mut writer = videoio::VideoWriter::new(
        "appsrc ! videoconvert ! videoscale
        ! video/x-raw,
        ! x264enc speed-preset=veryfast tune=zerolatency bitrate=800
        ! video/x-h264,profile=baseline
        ! rtspclientsink location=rtsp://localhost:8554/mystream",
        0,
        30.0,
        Size {
            width: 640,
            height: 480,
        },
        true,
    )?;

    if !writer.is_opened()? {
        eprintln!("Could not open GST writer.");
    }

    println!("Frame width: {}", cap.get(CAP_PROP_FRAME_WIDTH)?.round());
    println!("Frame height: {}", cap.get(CAP_PROP_FRAME_HEIGHT)?.round());

    let fps = cap.get(CAP_PROP_FPS)?;
    println!("FPS: {}", fps);

    let mut frame = Mat::default();
    let mut frame2 = Mat::default();

    loop {
        cap.read(&mut frame)?;

        for effect in effects.iter() {
            if effect.enabled.load(Ordering::Relaxed) {
                (effect.function)(&frame, &mut frame2)?;
                frame = frame2.clone();
            }
        }

        highgui::imshow("video capture", &frame)?;
        writer.write(&frame)?;

        if highgui::wait_key(1 as i32)? == 27 {
            break;
        }
    }
    highgui::destroy_all_windows()?;
    cap.release()?;
    Ok(())
}

fn main() {
    camera_in().unwrap();
}
