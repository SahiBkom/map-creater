use gpx::read;
use gpx::{Gpx, Track, TrackSegment};
use image::io::Reader as ImageReader;
use image::{GenericImage, ImageBuffer};
use image::{Rgba, RgbaImage};
use reqwest::header::USER_AGENT;
use reqwest::Client;
use std::fs::File;
use std::io::{BufReader, Cursor};
use tail_server_url::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This XML file actually exists — try it for yourself!
    let file = File::open("tests/data/track.gpx").unwrap();
    let reader = BufReader::new(file);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx: Gpx = read(reader).unwrap();

    // Each GPX file has multiple "tracks", this takes the first one.
    let track: &Track = &gpx.tracks[0];
    // assert_eq!(track.name, Some(String::from("Example GPX Document")));

    // Each track will have different segments full of waypoints, where a
    // waypoint contains info like latitude, longitude, and elevation.
    for segment in &track.segments {
        let mut x_min = f64::MAX;
        let mut y_min = f64::MAX;
        let mut x_max = f64::MIN;
        let mut y_max = f64::MIN;
        for point in &segment.points {
            x_min = x_min.min(point.point().x());
            y_min = y_min.min(point.point().y());
            x_max = x_max.max(point.point().x());
            y_max = y_max.max(point.point().y());
        }

        println!("{:?} {:?} {:?} {:?}", x_min, y_min, x_max, y_max);

        let iter = TailServerUrl::new_openstreetmap(12).deg_box(y_min, x_min, y_max, x_max);

        let (size_x, size_y) = iter.size();

        let origin_x = iter.origin_x() * 256;
        let origin_y = iter.origin_y() * 256;

        println!("size {:?}", iter.size());

        if (size_x * size_y) > 5 {
            println!("TO BIG");
            return Ok(());
        }

        let mut img: RgbaImage = ImageBuffer::new(size_x as u32 * 256, size_y as u32 * 256);

        for tail in iter {
            let resp = Client::builder()
                .build()?
                .get(tail.url())
                .header(
                    USER_AGENT,
                    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/113.0s",
                )
                .send()
                .await?;

            let bytes = resp.bytes().await?;

            let img2 = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?;

            img.copy_from(&img2, tail.x() * img2.width(), tail.y() * img2.height())?;
        }

        for point in &segment.points {
            let (x, y) = TailServerUrl::deg2num(point.point().y(), point.point().x(), 12 + 8);

            let xx = x - origin_x;
            let yy = y - origin_y;

            // TODO: create a line
            *img.get_pixel_mut(xx as u32, yy as u32) = Rgba::from([0u8, 0u8, 0u8, 0u8]);
        }

        img.save("test.png")?;
    }

    Ok(())
}
