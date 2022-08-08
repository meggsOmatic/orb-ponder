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
}


pub fn get_point_in_sphere(rng: &mut TraceContext) -> Vec3A {
  loop {
      //let r = Vec3A::from(rng.gen()) * Vec3A::splat(2.0) - Vec3A::ONE;
      let r = rng.rng3() * Vec3A::splat(2.0) - Vec3A::ONE;
      if r.length_squared() <= 1.0 { return r; }
  }
}



