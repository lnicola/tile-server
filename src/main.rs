use std::convert::Infallible;
use std::io;
use std::path::Path;

use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::{get, web, App, HttpServer};
use gdal::raster::Buffer;
use gdal::{Dataset, Driver};

use self::config::Config;
use self::error::Error;
use self::tile_grid::{Extent, TileGrid};

mod config;
mod error;
mod tile_grid;

#[get("tile/{file}/{z}/{x}/{y}.png")]
async fn index(
    web::Path((file, z, x, mut y)): web::Path<(String, u8, u32, u32)>,
    config: web::Data<Config>,
) -> Result<NamedFile, Error> {
    let file_name = format!("cache/{}_{}_{}_{}.png", file, z, x, y);
    let file_name_clone = file_name.clone();
    let exists =
        web::block::<_, _, Infallible>(move || Ok(Path::new(&file_name_clone).exists())).await?;
    if !exists {
        if config.reverse_y {
            y = (1 << z) - 1 - y;
        }

        let tile_extent = config.tile_grid.tile_extent(x, y, z);
        let dataset = web::block(move || Dataset::open(Path::new(&file))).await?;
        let geo_transform = dataset.geo_transform()?;
        let raster_size = dataset.raster_size();
        let (x_min, x_size, y_max, y_size) = (
            geo_transform[0],
            geo_transform[1],
            geo_transform[3],
            geo_transform[5],
        );
        dbg!(&geo_transform);
        let image_extent = Extent {
            xmin: x_min,
            ymin: y_max + y_size * raster_size.1 as f64,
            xmax: x_min + x_size * raster_size.0 as f64,
            ymax: y_max,
        };
        dbg!(&image_extent);
        let intersection_extent = Extent {
            xmin: tile_extent.xmin.max(image_extent.xmin),
            ymin: tile_extent.ymin.max(image_extent.ymin),
            xmax: tile_extent.xmax.min(image_extent.xmax),
            ymax: tile_extent.ymax.min(image_extent.ymax),
        };
        dbg!(&intersection_extent);
        if intersection_extent.xmin >= intersection_extent.xmax
            || intersection_extent.ymin >= intersection_extent.ymax
        {
            return Err(Error::OutsideBounds);
        }
        let px = (intersection_extent.xmin - image_extent.xmin) / x_size;
        let py = (intersection_extent.ymin - image_extent.ymax) / y_size;
        let px1 = (intersection_extent.xmax - image_extent.xmin) / x_size;
        let py1 = (intersection_extent.ymax - image_extent.ymax) / y_size;

        let src_width = (tile_extent.xmax - tile_extent.xmin) / x_size;
        let src_height = (tile_extent.ymin - tile_extent.ymax) / y_size;

        let src_tile_width_ratio = config.tile_width as f64 / src_width;
        let src_tile_height_ratio = config.tile_height as f64 / src_height;

        let off_left = (intersection_extent.xmin - tile_extent.xmin) / x_size;
        let off_top = (intersection_extent.ymax - tile_extent.ymax) / y_size;
        let off_right = (tile_extent.xmax - intersection_extent.xmax) / x_size;
        let off_bottom = (tile_extent.ymin - intersection_extent.ymin) / y_size;

        let off_left = off_left.round() as isize;
        let off_top = off_top.round() as isize;
        let off_right = off_right.round() as isize;
        let off_bottom = off_bottom.round() as isize;

        let win_x = px.round() as isize;
        let win_y = py1.round() as isize;
        let win_w = (px1 - px).round() as usize;
        let win_h = (py - py1).round() as usize;

        eprintln!(
            "{}/{}/{}\n({}, {})x({}, {}) {:?}",
            z,
            x,
            y,
            win_x,
            win_y,
            win_w,
            win_h,
            (off_left, off_top, off_right, off_bottom)
        );

        let ol = (off_left as f64 * src_tile_width_ratio).round() as usize;
        let ot = (off_top as f64 * src_tile_height_ratio).round() as usize;
        let or = (off_right as f64 * src_tile_width_ratio).round() as usize;
        let ob = (off_bottom as f64 * src_tile_height_ratio).round() as usize;

        let input_position = (win_x, win_y);
        let input_size = (win_w, win_h);
        let output_position = (ol as isize, ot as isize);
        let output_size = (config.tile_width - ol - or, config.tile_height - ot - ob);

        let file_name_clone = file_name.clone();
        web::block::<_, _, Error>(move || {
            let out = Driver::get("MEM")?.create(
                "",
                config.tile_width as isize,
                config.tile_height as isize,
                4,
            )?;
            let mut alpha = vec![255; output_size.0 * output_size.1];
            for i in 1..=3 {
                let buf = dataset.rasterband(i)?.read_as::<u8>(
                    input_position,
                    input_size,
                    output_size,
                )?;
                buf.data.iter().zip(alpha.iter_mut()).for_each(|(&p, a)| {
                    if p == 0 {
                        *a = 0;
                    }
                });
                out.rasterband(i)?
                    .write(output_position, output_size, &buf)?;
            }

            let buffer = Buffer::new(output_size, alpha);
            out.rasterband(4)?
                .write(output_position, output_size, &buffer)?;

            let png_driver = Driver::get("PNG")?;
            out.create_copy(&png_driver, &file_name_clone)?;
            Ok(())
        })
        .await?;
    }
    let file = NamedFile::open(file_name)?;
    Ok(file)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    std::fs::create_dir_all("cache")?;
    let config = Config {
        tile_grid: TileGrid::web_mercator(),
        reverse_y: false,
        tile_width: 256,
        tile_height: 256,
    };

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .wrap(Cors::default().send_wildcard().allowed_methods(vec!["GET"]))
            .service(index)
    })
    .bind("0.0.0.0:3011")?
    .run()
    .await
}
