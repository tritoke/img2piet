use clap::Parser;
use std::path::PathBuf;
use rayon::prelude::*;
use image::io::Reader as ImageReader;
use image::{Rgb, ImageBuffer};
use cached::proc_macro::cached;
use std::ffi::OsStr;

// sorted
const PIET_COLOURS: [Rgb<u8>; 20] = [
    Rgb([0xFF, 0xC0, 0xC0]), // light red
    Rgb([0xFF, 0xFF, 0xC0]), // light yellow
    Rgb([0xC0, 0xFF, 0xC0]), // light green
    Rgb([0xC0, 0xFF, 0xFF]), // light cyan
    Rgb([0xC0, 0xC0, 0xFF]), // light blue
    Rgb([0xFF, 0xC0, 0xFF]), // light magent
    Rgb([0xFF, 0x00, 0x00]), // red
    Rgb([0xFF, 0xFF, 0x00]), // yellow
    Rgb([0x00, 0xFF, 0x00]), // green
    Rgb([0x00, 0xFF, 0xFF]), // cyan
    Rgb([0x00, 0x00, 0xFF]), // blue
    Rgb([0xFF, 0x00, 0xFF]), // magent
    Rgb([0xC0, 0x00, 0x00]), // dark red
    Rgb([0xC0, 0xC0, 0x00]), // dark yellow
    Rgb([0x00, 0xC0, 0x00]), // dark green
    Rgb([0x00, 0xC0, 0xC0]), // dark cyan
    Rgb([0x00, 0x00, 0xC0]), // dark blue
    Rgb([0xC0, 0x00, 0xC0]), // dark magenta
    Rgb([0xFF, 0xFF, 0xFF]), // white
    Rgb([0x00, 0x00, 0x00]), // black
];

/// Simple program to convert any image to a valid piet program
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name(s) of the image to convert
    #[clap(short, long, use_value_delimiter = true)]
    images: Vec<PathBuf>,
}

fn main() {
    let Args { images } = Args::parse();

    let (oks, errs): (Vec<_>, Vec<_>) = images
        .into_par_iter()
        .map(|path| {
            let res = ImageReader::open(&path)
                .map_err(anyhow::Error::new)
                .and_then(|reader| reader.decode().map_err(anyhow::Error::new));
            match res {
                Ok(img) => (Some((path, img)), None),
                Err(e) => (None, Some((path, e))),
            }
        })
        .unzip();

    for (path, err) in errs.into_iter().filter(Option::is_some).map(Option::unwrap) {
        println!("Encountered an error while reading image {path:?} - {err}.");
    }

    oks.into_par_iter()
        .filter(Option::is_some)
        .map(Option::unwrap)
        .for_each(|(mut path, im)| {
            let height = im.height();
            let width = im.width();
            let mut buffer: Vec<u8> = im.into_rgb8().into_vec();
            buffer
                .par_chunks_mut(3)
                .for_each(|chunk| {
                    let Rgb([r, g, b]) = closest_piet_colour(Rgb([chunk[0], chunk[1], chunk[2]]));
                    chunk[0] = r;
                    chunk[1] = g;
                    chunk[2] = b;
                });
            let rgb_image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_vec(width, height, buffer).unwrap();

            //for pixel in rgb_image.pixels_mut().into_par_iter() {
            //    *pixel = closest_piet_colour(*pixel);
            //}

            let pc = path.clone();
            let (name, _) = pc
                .file_name()
                .and_then(OsStr::to_str)
                .unwrap()
                .split_once('.')
                .expect("Failed to get file name/suffix.");

            path.set_file_name(format!("{name}_piet.png"));
            rgb_image.save(path)
                .expect("Failed to save new image file.");
        });
}

#[cached]
fn closest_piet_colour(Rgb([r, g, b]): Rgb<u8>) -> Rgb<u8> {
    *PIET_COLOURS
        .iter()
        .min_by_key(|rgb| {
            let Rgb([ro, go, bo]) = rgb;
            let rd = r as i32 - *ro as i32;
            let gd = g as i32 - *go as i32;
            let bd = b as i32 - *bo as i32;

            rd.pow(2) + gd.pow(2) + bd.pow(2)
        })
        .unwrap()
}
