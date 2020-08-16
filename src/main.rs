use std::time::Instant;
use std::fs::File;
use std::io::prelude::*;

use crate::prelude::*;
use crate::reducers::auto_reduce_frame;

pub mod prelude;
pub mod reducers;
pub mod test;

fn main() -> std::io::Result<()> {
    let output_info = OutputInfo {
        version: 1,
        height: 480,
        width: 360,
        fps: 4,
    };

    let mut output: Bytevec = Vec::new();
    output.extend(&output_info.version.to_be_bytes());
    output.extend(&output_info.height.to_be_bytes());
    output.extend(&output_info.width.to_be_bytes());
    output.extend(&output_info.fps.to_be_bytes());

    let media_input = test::get_test_media();

    let start = Instant::now();
    for i in 0..media_input.len() {
        output.extend(auto_reduce_frame(&output_info, media_input.get(i).unwrap(),
            if i > 0 { media_input.get(i - 1) } else { None }, i == 0));
    }

    println!("Output len: {}", output.len());
    println!("Time per frame: {:?}", start.elapsed() / 4);

    let mut output_file = File::create("./test.out")?;
    output_file.write_all(&output)?;

    Ok(())
}
