
use crate::shapes::*;
use crate::geom::*;
use crate::scene::*;

use std::{io::Cursor, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use image::buffer::ConvertBuffer;
use image::io::Reader as ImageReader;
use image::*;
use glam::{ *, f32::* };
use quasirandom::Qrng;
use lerp::Lerp;
use rand::{Rng, rngs::ThreadRng, thread_rng};


pub trait Material : std::fmt::Debug {
  fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A;
}


#[derive(Debug)]
pub struct Lambertian(pub Vec3A);

impl Material for Lambertian {
  fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A {
      if !ctx.try_push() { return Vec3A::ZERO; }

      let color = self.0 * scene.get_color(
          Ray {
              origin: hit.world_pos,
              direction: (hit.world_normal + 0.999 * get_point_in_sphere(ctx)).normalize()
          },
          ctx);
      ctx.pop();
      color
  }
}


