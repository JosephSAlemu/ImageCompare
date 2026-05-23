use std::env;
use colored::Colorize;
use image::{GenericImage, GenericImageView, ImageReader, Rgba, DynamicImage};
mod gui;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("{}: {}", "Error".red(), "This program takes 2 images as arguments.");
        return;
    }

    let img_one: &str = &args[1];
    let img_two: &str = &args[2];

    let (output, centers) = compare_images(img_one, img_two);

    println!("{} red islands found.", centers.len());

    gui::launch(output, centers).expect("Failed to launch GUI");
}

// Sets non-matching pixels as red on first image. Also returns the centers of the red islands.
fn compare_images(img_one: &str, img_two: &str) -> (DynamicImage, Vec<(u32, u32)>) {
    let img_one_r = ImageReader::open(img_one)
        .expect("Failed to open image one")
        .decode()
        .expect("Failed to decode image one");

    let img_two_r = ImageReader::open(img_two)
        .expect("Failed to open image two")
        .decode()
        .expect("Failed to decode image two");

    if img_one_r.dimensions() != img_two_r.dimensions() {
        println!("{}: {}", "Error".red(), "The images have different dimensions.");
        std::process::exit(1);
    }

    let mut output = img_one_r.clone();

    for (x, y, pixel_one) in img_one_r.pixels() {
        let pixel_two = img_two_r.get_pixel(x, y);
        let [r, g, b, a] = output.get_pixel(x, y).0;

        if pixel_one != pixel_two {
            let new_r = ((r as f32 * 1.2) as u32).min(255) as u8;
            output.put_pixel(x, y, Rgba([new_r, g, b, a]));
        }
    }

    let centers = find_red_island_centers(&output, &img_one_r);

    (output, centers)
}

// BFS flood fill to find connected islands of redshifted pixels, returns their centers
fn find_red_island_centers(diff: &DynamicImage, original: &DynamicImage) -> Vec<(u32, u32)> {
    let (width, height) = diff.dimensions();
    let mut visited = vec![vec![false; height as usize]; width as usize];
    let mut centers = Vec::new();

    for start_x in 0..width {
        for start_y in 0..height {
            if visited[start_x as usize][start_y as usize] {
                continue;
            }
            if !is_redshifted(diff, original, start_x, start_y) {
                visited[start_x as usize][start_y as usize] = true;
                continue;
            }

            let mut queue = std::collections::VecDeque::new();
            let mut island_pixels: Vec<(u32, u32)> = Vec::new();

            queue.push_back((start_x, start_y));
            visited[start_x as usize][start_y as usize] = true;

            while let Some((cx, cy)) = queue.pop_front() {
                island_pixels.push((cx, cy));

                let neighbours = [
                    (cx.wrapping_sub(1), cy),
                    (cx + 1, cy),
                    (cx, cy.wrapping_sub(1)),
                    (cx, cy + 1),
                ];
                for (nx, ny) in neighbours {
                    if nx < width && ny < height
                        && !visited[nx as usize][ny as usize]
                        && is_redshifted(diff, original, nx, ny)
                    {
                        visited[nx as usize][ny as usize] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }

            let sum_x: u64 = island_pixels.iter().map(|(x, _)| *x as u64).sum();
            let sum_y: u64 = island_pixels.iter().map(|(_, y)| *y as u64).sum();
            let n = island_pixels.len() as u64;
            centers.push(((sum_x / n) as u32, (sum_y / n) as u32));
        }
    }

    centers
}

fn is_redshifted(diff: &DynamicImage, original: &DynamicImage, x: u32, y: u32) -> bool {
    let diff_r = diff.get_pixel(x, y).0[0];
    let orig_r = original.get_pixel(x, y).0[0];
    diff_r > orig_r
}
