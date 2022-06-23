use std::str::Chars;
use image::{DynamicImage, GenericImageView, Pixel};

type Matrix = Vec<Vec<f64>>;

pub fn find_distance(hash1: Chars, hash2: Chars) -> i32 {
    hash1.zip(hash2).fold(0,
                          |acc, x|
                              if x.0 != x.1
                              { acc + 1 } else { acc },
    )
}

pub fn find_hash(img: String) -> String {
    let size = 32;
    let img = image::open(img).unwrap()
        .resize(size, size, image::imageops::Lanczos3)
        .grayscale();
    println!("{:#?}", find_image_matrix(img));
    "".to_string()
}

fn find_image_matrix(img: DynamicImage) -> Matrix {
    let (_, _, x_size, y_size) = img.bounds();
    let mut matrix: Matrix = Vec::new();
    for x in 0..x_size {
        matrix.push(Vec::new());
        for y in 0..y_size {
            matrix[x as usize].push(find_xy_value(img.clone(), x, y));
        }
    }
    matrix
}

fn find_xy_value(img: DynamicImage, x: u32, y: u32) -> f64 {
    img.get_pixel(x, y).to_rgba().0[2] as f64
}

#[cfg(test)]
mod phash_tests {
    #[test]
    fn find_distance_test() {
        use super::*;
        assert_eq!(find_distance("1101".chars(), "1011".chars()), 2);
        assert_eq!(find_distance("1111".chars(), "1111".chars()), 0);
    }
}