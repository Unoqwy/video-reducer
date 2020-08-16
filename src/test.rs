use std::path::Path;

use crate::prelude::*;

pub fn get_test_media() -> Vec<Frame> {
    let mut frames = Vec::new();
    for i in 1..=4 {
        frames.push(get_test_frame(&i.to_string()));
    }
    frames
}

pub fn get_test_frame(name: &str) -> Frame {
    let img = image::open(Path::new(&format!("./assets/test_frame{}.jpg", name)));
    let mut frame = Vec::new();
    for pixel in img.unwrap().as_rgb8().unwrap().pixels() {
        frame.push([pixel.0[0], pixel.0[1], pixel.0[2]]);
    }
    frame
}
