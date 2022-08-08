
use crate::materials::*;
use crate::shapes::*;
use crate::geom::*;

use std::{io::Cursor, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use image::buffer::ConvertBuffer;
use image::io::Reader as ImageReader;
use image::*;
use glam::{ *, f32::* };
use lerp::Lerp;
use quasirandom::*;
use rand::{Rng, rngs::ThreadRng, thread_rng};

pub struct Scene<'a> {
  pub shapes: Vec<Box<dyn 'a + Shape>>
}

impl<'a> Scene<'a> {
  pub fn get_color(&self, ray: Ray, ctx: &mut TraceContext) -> Vec3A {
      let mut best_hit : Option<Hit> = None;
      for shape in &self.shapes {
          if let Some(hit) = shape.trace_ray(ray) {
              if !hit.started_inside && hit.distance > 0.0001 &&
                  (best_hit.is_none() || hit.distance < best_hit.unwrap().distance) {
                  best_hit = Some(hit);
              }
          }
      }

      if let Some(hit) = best_hit {
          let color = hit.material.get_color(self, ray, &hit, ctx);
          color
      } else {
          1.0 * Vec3A::new(0.1, 0.2, 0.3) + 0.04 * ray.direction.dot(Vec3A::new(0.8, 1.2, 1.6).normalize()).max(0.).powf(10.) * Vec3A::new(200., 175., 150.)
      }
  }
}


pub struct RngSet<T : Quasirandom + FromUniform> {
  next_entry: usize,
  list: Vec<Qrng<T>>
}



pub struct TraceContext {
  current_depth: i32,
  max_depth: i32,

  next_rng1: usize,
  next_rng2: usize,
  next_rng3: usize,
  rng1_list: Vec<Qrng<f32>>,
  rng2_list: Vec<Qrng<(f32, f32)>>,
  rng3_list: Vec<Qrng<(f32, f32, f32)>>,
  thread_rng: ThreadRng,
  reseed: f64,
}

impl TraceContext {
  pub fn new(max_depth: i32) -> TraceContext {
    TraceContext {
      current_depth: 0,
      max_depth,
      next_rng1: 0,
      next_rng2: 0,
      next_rng3: 0,
      rng1_list: Vec::new(),
      rng2_list: Vec::new(),
      rng3_list: Vec::new(),
      thread_rng: thread_rng(),
      reseed: thread_rng().gen(),
    }
  }

  #[inline(always)]
  pub fn try_push(&mut self) -> bool {
    assert!(self.current_depth >= 0);
    assert!(self.current_depth <= self.max_depth);
    if self.current_depth == self.max_depth {
      false
    } else {
      self.current_depth += 1;
      true
    }
  }

  #[inline(always)]
  pub fn pop(&mut self) {
    assert!(self.current_depth > 0);
    assert!(self.current_depth <= self.max_depth);
    self.current_depth -= 1;
  }


  #[inline(always)]
  pub fn rngen(&mut self) -> f32 {
    self.thread_rng.gen()
  }


  #[inline(always)]
  pub fn rng1(&mut self) -> f32 {
    if self.next_rng1 >= self.rng1_list.len() {
      self.next_rng1 = self.rng1_list.len();
      let seed = ((self.next_rng1 + 1) as f64 + self.reseed).exp().fract();
      self.rng1_list.push(Qrng::<f32>::new(seed));
    }
    self.next_rng1 += 1;
    self.rng1_list[self.next_rng1 - 1].gen()
  }

  #[inline(always)]
  pub fn rng2(&mut self) -> Vec2 {
    if self.next_rng2 >= self.rng2_list.len() {
      self.next_rng2 = self.rng2_list.len();
      let seed = ((self.next_rng2 + 1) as f64 + self.reseed).exp().fract();
      self.rng2_list.push(Qrng::<(f32, f32)>::new(seed));
    }
    self.next_rng2 += 1;
    Vec2::from(self.rng2_list[self.next_rng2 - 1].gen())
  }

  #[inline(always)]
  pub fn rng3(&mut self) -> Vec3A {
    if self.next_rng3 >= self.rng3_list.len() {
      self.next_rng3 = self.rng3_list.len();
      let seed = ((self.next_rng3 + 1) as f64 + self.reseed).exp().fract();
      self.rng3_list.push(Qrng::<(f32, f32, f32)>::new(seed));
    }
    self.next_rng3 += 1;
    Vec3A::from(self.rng3_list[self.next_rng3 - 1].gen())
  }

  pub fn next_sample(&mut self) {
    assert_eq!(self.current_depth, 0);
    self.next_rng1 = 0;
    self.next_rng2 = 0;
    self.next_rng3 = 0;
  }

  pub fn next_pixel(&mut self) {
    assert_eq!(self.current_depth, 0);
    self.next_rng1 = 0;
    self.next_rng2 = 0;
    self.next_rng3 = 0;
    self.rng1_list.clear();
    self.rng2_list.clear();
    self.rng3_list.clear();
    self.reseed = self.thread_rng.gen();
  }

  #[inline(always)]
  pub fn blur_vector(&mut self, v: Vec3A, blur_amount: f32) -> Vec3A {
    ((Vec3A::splat(2.0) * self.rng3() - Vec3A::ONE) * blur_amount.clamp(0.0, 0.999) + v).normalize()
  }
}

