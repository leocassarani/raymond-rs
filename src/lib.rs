extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
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
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    fn subtract(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
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
    fn red() -> Self {
        Self::new(255., 0., 0.)
    }

    fn green() -> Self {
        Self::new(0., 255., 0.)
    }

    fn blue() -> Self {
        Self::new(0., 0., 255.)
    }

    fn black() -> Self {
        Self::new(0., 0., 0.)
    }

    fn new(red: f64, green: f64, blue: f64) -> Self {
        Self { red, green, blue }
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

    fn intersects(&self, ray: &Ray) -> bool {
        let oc = ray.origin.subtract(&self.center);
        ray.direction.dot(&oc).powi(2) >= oc.length().powi(2) - self.radius.powi(2)
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
}

#[wasm_bindgen]
pub struct Scene {
    camera: Camera,
    spheres: Vec<Sphere>,
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
            Sphere::new(Vec3::new(5., 3., 5.), 2., RGB::red()),
            Sphere::new(Vec3::new(1., 5., 10.), 2., RGB::green()),
        ];

        Self { camera, spheres }
    }

    pub fn render(&self, img: &mut Image) {
        for y in 0..img.height {
            let y_offset = y as f64 / img.height as f64;

            for x in 0..img.width {
                let x_offset = x as f64 / img.width as f64;
                let ray = self.camera.cast(x_offset, y_offset);

                for sphere in &self.spheres {
                    if sphere.intersects(&ray) {
                        img.draw(x, y, sphere.color);
                        break;
                    }

                    img.draw(x, y, RGB::black());
                }
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
    fn draw(&mut self, x: usize, y: usize, color: RGB) {
        let idx = 4 * (x + y * self.width);
        self.pixels[idx] = color.red as u8;
        self.pixels[idx + 1] = color.green as u8;
        self.pixels[idx + 2] = color.blue as u8;
        self.pixels[idx + 3] = 255;
    }
}
