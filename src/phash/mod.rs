#[allow(dead_code)]
use std::f64::consts::PI;
use std::ops::Add;
use std::str::Chars;
use image::{DynamicImage, GenericImageView, Pixel};

type Matrix = Vec<Vec<f64>>;

struct DctPoint<'a> {
    x_max: i64,
    y_max: i64,
    x_scales: &'a mut [f64; 2],
    y_scales: &'a mut [f64; 2],
}

impl DctPoint<'_> {
    fn calculate(self: &Self, image_data: &Matrix, x: i64, y: i64) -> f64 {
        let mut sum = 0.;
        for i in 0..self.x_max {
            for j in 0..self.y_max {
                let image_value = image_data[i as usize][j as usize];
                let fst_cosine = ((((1 + (2 * i)) * x) as f64) * PI / (2. * self.x_max as f64)).cos();
                let snd_cosine = ((((1 + (2 * j)) * y) as f64) * PI / (2. * self.y_max as f64)).cos();
                sum += image_value * fst_cosine * snd_cosine;
            }
        }
        sum * self.find_scale_factor(x, y)
    }

    fn find_scale_factor(self: &Self, x: i64, y: i64) -> f64 {
        let mut x_scale_factor = self.x_scales[1];
        if x == 0 {
            x_scale_factor = self.x_scales[0];
        }
        let mut y_scale_factor = self.y_scales[1];
        if y == 0 {
            y_scale_factor = self.y_scales[0];
        }
        x_scale_factor * y_scale_factor
    }
}

pub fn find_distance(hash1: &Chars, hash2: &Chars) -> i32 {
    hash1.clone().zip(hash2.clone()).fold(0,
                          |acc, x|
                              if x.0 != x.1
                              { acc + 1 } else { acc },
    )
}

pub fn find_hash(img: String) -> String {
    let size = 32;
    let img = image::open(img).unwrap()
        .resize_to_fill(size, size, image::imageops::Lanczos3)
        .grayscale();
    let image_matrix = find_image_matrix(img);
    let small_matrix = reduce_matrix(image_matrix, 8);
    let dct_matrix = find_dct_matrix(small_matrix);
    let dct_mean_value = calculate_mean_value(&dct_matrix);
    build_hash(dct_matrix, dct_mean_value)
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
    img.get_pixel(x, y).to_bgr().0[0] as f64
}

fn find_dct_matrix(matrix: Matrix) -> Matrix {
    let x_max = matrix.len();
    let y_max = matrix[0].len();
    let dct_point = &DctPoint {
        x_max: x_max as i64,
        y_max: y_max as i64,
        x_scales: &mut [1. / (x_max as f64).sqrt(), (2. / x_max as f64).sqrt()],
        y_scales: &mut [1. / (y_max as f64).sqrt(), (2. / y_max as f64).sqrt()],
    };
    let mut dct_matrix: Matrix = Vec::new();
    for x in 0..x_max {
        dct_matrix.push(Vec::new());
        for y in 0..y_max {
            dct_matrix[x].push(dct_point.calculate(&matrix, x as i64, y as i64));
        }
    }
    dct_matrix
}

fn reduce_matrix(dct_matrix: Matrix, size: i64) -> Matrix {
    let mut new_matrix: Matrix = Vec::new();
    for x in 0..size {
        new_matrix.push(Vec::new());
        for y in 0..size {
            new_matrix[x as usize].push(dct_matrix[x as usize][y as usize]);
        }
    }
    new_matrix
}

fn calculate_mean_value(dct_matrix: &Matrix) -> f64 {
    let mut avg = 0.;
    let n = dct_matrix.len();
    for x in 0..n {
        for y in (x+1)..n {
            avg += dct_matrix[x][y] / (n * n) as f64;
        }
    }
    avg
}

fn build_hash(dct_matrix: Matrix, dct_mean_value: f64) -> String {
    let mut hash = String::new();
    let x_size = dct_matrix.len();
    let y_size = dct_matrix[0].len();
    for x in 0..x_size {
        for y in 0..y_size {
            if dct_matrix[x][y] > dct_mean_value {
                hash = hash.add("1");
            } else {
                hash = hash.add("0")
            }
        }
    }
    hash
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