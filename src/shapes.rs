#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::materials::*;
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



pub trait Shape : std::fmt::Debug + dyn_clone::DynClone  {
  fn trace_ray(&self, ray: Ray) -> Option<Hit>;
  fn get_bounds(&self) -> Option<(Vec3A, Vec3A)>;
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere<'a> {
  pub center: Vec3A,
  pub radius: f32,
  pub material: &'a dyn Material
}

impl<'a> Sphere<'a> {
  pub fn intersect(&self, r: Ray) -> Option<(f32, f32)> {
      let to_center = self.center - r.origin;
      let dir_dist_to_center = to_center.dot(r.direction);
      let dir_to_center = dir_dist_to_center * r.direction;
      let perp_to_center = to_center - dir_to_center;
      let perp_len_squared = perp_to_center.length_squared();
      if perp_len_squared < self.radius * self.radius {
          let off = (self.radius * self.radius - perp_len_squared).sqrt();
          Some((dir_dist_to_center - off, dir_dist_to_center + off))
      } else {
          None
      }
  }
}

impl<'a> Shape for Sphere<'a> {
  fn trace_ray(&self, ray: Ray) -> Option<Hit> {
      if let Some(hit_range) = self.intersect(ray) {
          if hit_range.1 > 0. {
              let started_inside = hit_range.0 < 0.;
              let distance = if started_inside { hit_range.1 } else { hit_range.0 };
              let world_pos = ray.at(distance);
              let local_pos = world_pos - self.center;
              let normal = local_pos.normalize_or_zero();
              Some(Hit {
                  world_pos,
                  world_normal: normal,
                  local_pos,
                  local_normal: normal,
                  material: self.material,
                  distance,
                  started_inside
              })
          } else {
              None
          }
      } else {
          None
      }
  }

  fn get_bounds(&self) -> Option<(Vec3A, Vec3A)> {
      Some((self.center - Vec3A::splat(self.radius), self.center - Vec3A::splat(self.radius)))
  }
}



#[derive(Debug, Clone, Copy)]
pub struct Plane<'a> {
  pub normal: Vec3A,
  pub center: Vec3A,
  pub material: &'a dyn Material,
  n_dot_c: f32,
  right: Vec3A,
  up: Vec3A
}


impl<'a> Plane<'a> {
  pub fn new(normal: Vec3A, right: Vec3A, center: Vec3A, material: &'a dyn Material) -> Plane<'a> {
    let up = normal.cross(right);
    let right = up.cross(normal).normalize();
    let up = up.normalize();
    let normal = normal.normalize();
    Plane { normal, center, right, up, n_dot_c: normal.dot(center), material }
  }
}


impl<'a> Shape for Plane<'a> {
  fn trace_ray(&self, ray: Ray) -> Option<Hit> {
    let n_dot_dir = self.normal.dot(ray.direction);
    let n_dot_o = self.normal.dot(ray.origin);
    //dbg!(ray, n_dot_dir, n_dot_o);
    if n_dot_dir.abs() > 0.000001 && n_dot_o.abs() > 0.000001 {
      let dist = (self.n_dot_c - n_dot_o) / n_dot_dir;
      if dist > 0. {
        let world_pos = ray.at(dist);
        let local_offset = world_pos - self.center;
        //dbg!(dist, world_pos, local_offset);
        return Some(Hit {
          world_pos,
          world_normal: if n_dot_o >= 0. { self.normal } else { -self.normal },
          local_pos: Vec3A::new(self.right.dot(local_offset), self.up.dot(local_offset), 0.),
          local_normal: if n_dot_o >= 0.0 { Vec3A::Z } else { Vec3A::NEG_Z },
          material: self.material,
          distance: dist,
          started_inside: false
        })
      }
    }
    None
}

  fn get_bounds(&self) -> Option<(Vec3A, Vec3A)> { None }
}