use crate::materials::*;
use crate::shapes::*;
use crate::scene::*;

use std::{io::Cursor, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use image::buffer::ConvertBuffer;
use image::io::Reader as ImageReader;
use image::*;
use glam::{ *, f32::* };
use quasirandom::Qrng;
use lerp::Lerp;
use rand::{Rng, rngs::ThreadRng, thread_rng};


#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A
}

impl Ray {
    pub fn at(&self, dist: f32) -> Vec3A {
        self.origin + dist * self.direction
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Hit<'a> {
    pub world_pos: Vec3A,
    pub world_normal: Vec3A,
    pub local_pos: Vec3A,
    pub local_normal: Vec3A,
    pub material: &'a dyn Material,
    pub distance: f32,
    pub started_inside: bool,
    pub local_to_world: Mat3A
}


pub fn get_point_in_sphere(rng: &mut TraceContext) -> Vec3A {
  loop {
      //let r = Vec3A::from(rng.gen()) * Vec3A::splat(2.0) - Vec3A::ONE;
      let r = rng.rng3() * Vec3A::splat(2.0) - Vec3A::ONE;
      if r.length_squared() <= 1.0 { return r; }
  }
}


pub fn reflect(ray: Vec3A, normal: Vec3A) -> Vec3A {
  ray - 2. * ray.dot(normal) * normal
}


pub fn linear_to_gamma_1(c: f32) -> f32 {
    if c > 0.0 {
        if c <= 0.0031308 { c * 12.92 }
        else if c < 1.0 { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
        else { 1.0 }
    } else { 0.0 }
}

pub fn gamma_to_linear_1(c: f32) -> f32 {
    if c > 0.0 {
        if c <= 0.04045 { c / 12.92 }
        else if c < 1.0 { ((c + 0.055) / 1.055).powf(2.4) }
        else { 1.0 }
    } else { 0.0 }
}

pub fn gamma_to_linear_rgb(rgb: Rgb<f32>) -> Vec3 {
    vec3(gamma_to_linear_1(rgb[0]), gamma_to_linear_1(rgb[1]), gamma_to_linear_1(rgb[2]))
}

pub fn linear_to_gamma_rgb(rgb: Vec3) -> Rgb<f32> {
    Rgb([linear_to_gamma_1(rgb[0]), linear_to_gamma_1(rgb[1]), linear_to_gamma_1(rgb[2])])
}

