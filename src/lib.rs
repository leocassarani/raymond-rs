extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use std::f64;
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
        let len = self.length();
        Vec3::new(self.x / len, self.y / len, self.z / len)
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
        Self::new(255., 0., 0.)
    }

    #[allow(dead_code)]
    fn green() -> Self {
        Self::new(0., 255., 0.)
    }

    #[allow(dead_code)]
    fn blue() -> Self {
        Self::new(0., 0., 255.)
    }

    #[allow(dead_code)]
    fn black() -> Self {
        Self::new(0., 0., 0.)
    }

    fn new(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
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
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    fn point_at(&self, t: f64) -> Vec3 {
        self.origin.add(&self.direction.scale(t))
    }
}

struct Sphere {
    center: Vec3,
    radius: f64,
    color: RGB,
}

impl Sphere {
    fn new(center: Vec3, radius: f64, color: RGB) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }

    fn intersect(&self, ray: &Ray) -> Option<f64> {
        let oc = ray.origin.subtract(&self.center);
        let dot = ray.direction.dot(&oc);
        let sqrt_term = dot.sqr() - oc.length().sqr() + self.radius.sqr();

        if sqrt_term < 0. {
            None
        } else {
            let sqrt = sqrt_term.sqrt();
            vec![-dot - sqrt, -dot + sqrt]
                .iter()
                .cloned()
                .find(|&t| t >= 0.)
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

    fn illuminate(&self, point: &Vec3, surface_normal: &Vec3) -> f64 {
        let ray = self.pos.subtract(point);
        let cosine = surface_normal.dot(&ray.unit()) / surface_normal.length();
        self.power * cosine / (4. * f64::consts::PI * ray.length().sqr())
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
            Vec3::new(3., 3., 0.),
            Film::new(Vec3::new(0., 0., 3.), 6., 6.),
        );

        let spheres = vec![
            Sphere::new(Vec3::new(2., 6., 8.), 1., RGB::red()),
            Sphere::new(Vec3::new(1., 6., 5.), 1., RGB::blue()),
            Sphere::new(Vec3::new(3., 0., 12.), 5., RGB::green()),
        ];

        let lights = vec![
            Light::new(Vec3::new(1., 8., 0.), 300.),
            Light::new(Vec3::new(8., 5., 5.), 300.),
        ];

        Self {
            camera,
            spheres,
            lights,
        }
    }

    pub fn render(&self, img: &mut Image) {
        for y in 0..img.height {
            let y_offset = y as f64 / img.height as f64;

            for x in 0..img.width {
                let x_offset = x as f64 / img.width as f64;
                let ray = self.camera.cast(x_offset, y_offset);

                let nearest = self.spheres.iter().fold((None, f64::INFINITY), |min, s| {
                    match s.intersect(&ray) {
                        Some(t) if t < min.1 => (Some(s), t),
                        _ => min,
                    }
                });

                match nearest {
                    (Some(sphere), t) => {
                        let point = ray.point_at(t);
                        let normal = sphere.surface_normal(&point);

                        let power = self
                            .lights
                            .iter()
                            .map(|light| light.illuminate(&point, &normal))
                            .sum();

                        let color = sphere.color.shade(power);
                        img.draw(x, y, &color);
                    }
                    (None, _) => img.draw(x, y, &RGB::black()),
                };
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
        let idx = 4 * (x + y * self.width);
        self.pixels[idx] = color.red as u8;
        self.pixels[idx + 1] = color.green as u8;
        self.pixels[idx + 2] = color.blue as u8;
        self.pixels[idx + 3] = 255;
    }
}
