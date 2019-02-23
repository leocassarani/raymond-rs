extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use std::{f64, u8};
use wasm_bindgen::prelude::*;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

trait Square {
    fn sqr(self) -> Self;
}

impl Square for f64 {
    fn sqr(self) -> f64 {
        self.powi(2)
    }
}

#[derive(Copy, Clone)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    fn unit(&self) -> Vec3 {
        // It is faster to multiply than divide two numbers, so we turn the division
        // operations into multiplications by the inverse of the vector's length.
        let inv = 1. / self.length();
        Vec3::new(self.x * inv, self.y * inv, self.z * inv)
    }

    fn length(&self) -> f64 {
        (self.x.sqr() + self.y.sqr() + self.z.sqr()).sqrt()
    }

    fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn subtract(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn scale(&self, f: f64) -> Vec3 {
        Vec3::new(self.x * f, self.y * f, self.z * f)
    }

    fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

#[derive(Copy, Clone)]
struct RGB {
    red: f64,
    green: f64,
    blue: f64,
}

impl RGB {
    #[allow(dead_code)]
    fn red() -> Self {
        Self::new(1., 0., 0.)
    }

    #[allow(dead_code)]
    fn green() -> Self {
        Self::new(0., 1., 0.)
    }

    #[allow(dead_code)]
    fn blue() -> Self {
        Self::new(0., 0., 1.)
    }

    #[allow(dead_code)]
    fn black() -> Self {
        Self::new(0., 0., 0.)
    }

    #[allow(dead_code)]
    fn white() -> Self {
        Self::new(1., 1., 1.)
    }

    fn new(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
    }

    fn add(&self, other: &RGB) -> RGB {
        RGB::new(
            self.red + other.red,
            self.green + other.green,
            self.blue + other.blue,
        )
    }

    fn shade(&self, f: f64) -> RGB {
        if f <= 0. {
            RGB::black()
        } else if f >= 1. {
            self.clone()
        } else {
            RGB::new(self.red * f, self.green * f, self.blue * f)
        }
    }

    fn write(&self, pixels: &mut [u8]) {
        let max = u8::MAX as f64;
        let red = max * f64::min(self.red, 1.0);
        let green = max * f64::min(self.green, 1.0);
        let blue = max * f64::min(self.blue, 1.0);

        pixels[0] = red.round() as u8;
        pixels[1] = green.round() as u8;
        pixels[2] = blue.round() as u8;
        pixels[3] = u8::MAX;
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn cast(from: &Vec3, to: &Vec3) -> Self {
        let direction = to.subtract(from);
        Ray::new(from.clone(), direction)
    }

    fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    fn length(&self) -> f64 {
        self.direction.length()
    }

    fn unit(&self) -> Ray {
        Ray::new(self.origin, self.direction.unit())
    }

    fn point_at(&self, t: f64) -> Vec3 {
        self.origin.add(&self.direction.scale(t))
    }

    fn reflect(&self, point: &Vec3, normal: &Vec3) -> Ray {
        let cosine = self.direction.dot(&normal);
        let reflection = self.direction.subtract(&normal.scale(2. * cosine));
        Ray::new(point.clone(), reflection)
    }
}

struct Sphere {
    center: Vec3,
    radius: f64,
    color: RGB,
    glossiness: f64,
}

impl Sphere {
    fn new(center: Vec3, radius: f64, color: RGB, glossiness: f64) -> Self {
        Self {
            center,
            radius,
            color,
            glossiness,
        }
    }

    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let oc = ray.origin.subtract(&self.center);
        let dot = ray.direction.unit().dot(&oc);
        let sqrt_term = dot.sqr() - (oc.length().sqr() - self.radius.sqr());

        if sqrt_term < 0. {
            None
        } else {
            let sqrt = sqrt_term.sqrt();
            vec![-dot - sqrt, -dot + sqrt]
                .iter()
                .cloned()
                .find(|&t| t >= 1e-10)
        }
    }

    fn surface_normal(&self, point: &Vec3) -> Vec3 {
        point.subtract(&self.center)
    }
}

struct Light {
    pos: Vec3,
    power: f64,
}

impl Light {
    fn new(pos: Vec3, power: f64) -> Self {
        Self { pos, power }
    }

    fn illuminate(&self, spheres: &[Sphere], point: &Vec3, surface_normal: &Vec3) -> f64 {
        let ray = Ray::cast(point, &self.pos);
        let len = ray.length();
        let unit_ray = ray.unit();

        for sphere in spheres {
            if let Some(t) = sphere.intersect(&unit_ray) {
                if t < len {
                    return 0.;
                }
            }
        }

        let cosine = surface_normal.dot(&unit_ray.direction) / surface_normal.length();
        (self.power * cosine) / (4. * f64::consts::PI * len.sqr())
    }
}

struct Film {
    origin: Vec3,
    width: f64,
    height: f64,
}

impl Film {
    fn new(origin: Vec3, width: f64, height: f64) -> Self {
        Self {
            origin,
            width,
            height,
        }
    }

    fn project(&self, x: f64, y: f64) -> Vec3 {
        Vec3::new(
            self.origin.x + self.width * x,
            self.origin.y + self.height - (self.height * y),
            self.origin.z,
        )
    }
}

enum Move {
    Left,
    Right,
    Up,
    Down,
    Forward,
    Back,
}

struct Camera {
    eye: Vec3,
    film: Film,
}

impl Camera {
    fn new(eye: Vec3, film: Film) -> Self {
        Self { eye, film }
    }

    fn cast(&self, x: f64, y: f64) -> Ray {
        let origin = self.eye;
        let direction = self.film.project(x, y).subtract(&origin).unit();
        Ray::new(origin, direction)
    }

    fn move_one(&mut self, mov: Move) {
        match mov {
            Move::Left => {
                self.eye.x -= 1.;
                self.film.origin.x -= 1.;
            }
            Move::Right => {
                self.eye.x += 1.;
                self.film.origin.x += 1.;
            }
            Move::Up => {
                self.eye.y += 1.;
                self.film.origin.y += 1.;
            }
            Move::Down => {
                self.eye.y -= 1.;
                self.film.origin.y -= 1.;
            }
            Move::Forward => {
                self.eye.z += 1.;
                self.film.origin.z += 1.;
            }
            Move::Back => {
                self.eye.z -= 1.;
                self.film.origin.z -= 1.;
            }
        }
    }
}

#[wasm_bindgen]
pub struct Scene {
    camera: Camera,
    spheres: Vec<Sphere>,
    lights: Vec<Light>,
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let camera = Camera::new(
            Vec3::new(0., 0., -6.),
            Film::new(Vec3::new(-4., -3., 0.), 8., 4.5),
        );

        let spheres = vec![
            Sphere::new(Vec3::new(-1., 4., 15.), 2., RGB::red(), 1.),
            Sphere::new(Vec3::new(2., 2., 20.), 5., RGB::green(), 1.),
            Sphere::new(Vec3::new(10., -1., 25.), 3., RGB::new(0.5, 0., 0.5), 0.7),
            Sphere::new(Vec3::new(12., 4., 24.), 2., RGB::new(1., 1., 0.), 0.5),
            Sphere::new(Vec3::new(-5., -2., 12.), 3., RGB::blue(), 0.7),
            Sphere::new(Vec3::new(-1., -1., 11.), 1., RGB::new(1., 0.5, 0.7), 0.2),
            Sphere::new(Vec3::new(-11., 6., 12.), 4., RGB::white(), 1.),
            Sphere::new(Vec3::new(6., -9., 12.), 5., RGB::black(), 1.),
        ];

        let lights = vec![
            Light::new(Vec3::new(-3., 12., -2.), 3700.),
            Light::new(Vec3::new(12., 12., 22.), 1250.),
            Light::new(Vec3::new(-5., 8., 30.), 2500.),
        ];

        Self {
            camera,
            spheres,
            lights,
        }
    }

    pub fn render(&self, img: &mut Image) {
        let height_inv = 1. / img.height as f64;
        let width_inv = 1. / img.width as f64;

        for y in 0..img.height {
            let y_offset = y as f64 * height_inv;

            for x in 0..img.width {
                let x_offset = x as f64 * width_inv;
                let ray = self.camera.cast(x_offset, y_offset);

                let color = self.light(&ray, 1);
                img.draw(x, y, &color);
            }
        }
    }

    #[wasm_bindgen(js_name = moveLeft)]
    pub fn move_left(&mut self) {
        self.camera.move_one(Move::Left);
    }

    #[wasm_bindgen(js_name = moveRight)]
    pub fn move_right(&mut self) {
        self.camera.move_one(Move::Right);
    }

    #[wasm_bindgen(js_name = moveUp)]
    pub fn move_up(&mut self) {
        self.camera.move_one(Move::Up);
    }

    #[wasm_bindgen(js_name = moveDown)]
    pub fn move_down(&mut self) {
        self.camera.move_one(Move::Down);
    }

    #[wasm_bindgen(js_name = moveForward)]
    pub fn move_forward(&mut self) {
        self.camera.move_one(Move::Forward);
    }

    #[wasm_bindgen(js_name = moveBack)]
    pub fn move_back(&mut self) {
        self.camera.move_one(Move::Back);
    }
}

impl Scene {
    fn light(&self, ray: &Ray, depth: u8) -> RGB {
        let nearest =
            self.spheres
                .iter()
                .fold((None, f64::INFINITY), |min, s| match s.intersect(ray) {
                    Some(t) if t < min.1 => (Some(s), t),
                    _ => min,
                });

        match nearest {
            (Some(sphere), t) => {
                let point = ray.point_at(t);
                let normal = sphere.surface_normal(&point);

                let radiance = self
                    .lights
                    .iter()
                    .map(|light| light.illuminate(&self.spheres, &point, &normal))
                    .sum();

                let mut color = sphere.color;

                if sphere.glossiness > 0. && depth < 100 {
                    let reflection = ray.reflect(&point, &normal.unit());
                    let reflection_color =
                        self.light(&reflection, depth + 1).shade(sphere.glossiness);

                    color = color.add(&reflection_color)
                }

                color.shade(radiance)
            }
            (None, _) => {
                let y = 0.7 - ray.direction.y.abs();
                let mut x = ray.direction.x / 2.0;
                if x < y {
                    x = y
                }
                RGB::new(x, y, x)
            }
        }
    }
}

#[wasm_bindgen]
pub struct Image {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

#[wasm_bindgen]
impl Image {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Self {
        let len = (width * height) << 2;
        let pixels = vec![0; len];

        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn pixels(&self) -> *const u8 {
        self.pixels.as_ptr()
    }
}

impl Image {
    fn draw(&mut self, x: usize, y: usize, color: &RGB) {
        let idx = (x + y * self.width) << 2;
        color.write(&mut self.pixels[idx..idx + 4]);
    }
}
