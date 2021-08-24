extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<(), ffmpeg::Error> {
    ffmpeg::init().unwrap();
    let file = "./data/random_ball.mp4";
    if let Ok(mut ictx) = input(&file) {
        let input = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let mut decoder = input.codec().decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let mut frame_index = 0;

        let mut receive_and_process_decoded_frames =
            |decoder: &mut ffmpeg::decoder::Video| -> Result<(), ffmpeg::Error> {
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;
                    save_file(&rgb_frame, frame_index).unwrap();
                    frame_index += 1;
                }
                Ok(())
            };

        let mut packet_index = 0;
        let mut stream_index = 0;
        for (stream, packet) in ictx.packets() {
            packet_index += 1;
            println!("packet {}", packet_index);
            if stream.index() == video_stream_index {
                stream_index += 1;
                println!("index {}", stream_index);
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    Ok(())
}

fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    // println!("{:?}", frame.data(0));
    // let mut file = File::create(format!("data/raw/frame{}.ppm", index))?;
    // file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    // file.write_all(frame.data(0))?;

    let rgb_image =
        image::RgbImage::from_vec(frame.width(), frame.height(), frame.data(0).to_vec()).unwrap();

    // rgb_image
    //     .save(format!("data/raw/frame{}.png", index))
    //     .unwrap();
    let data = rgb_image.to_vec();
    let mut file = File::create(format!("data/raw/frame{}.ppm", index))?;
    file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    file.write_all(&data)?;

    Ok(())
}
