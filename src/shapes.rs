#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::geom::*;
use crate::materials::*;
use crate::scene::*;

use glam::{f32::*, *};
use image::buffer::ConvertBuffer;
use image::io::Reader as ImageReader;
use image::*;
use lerp::Lerp;
use quasirandom::Qrng;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Cursor,
};

pub trait Shape: std::fmt::Debug + dyn_clone::DynClone + Sync {
    fn trace_ray(&self, ray: Ray) -> Option<Hit>;
    fn get_bounds(&self) -> Option<(Vec3A, Vec3A)>;
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere<'a> {
    pub center: Vec3A,
    pub radius: f32,
    pub material: &'a dyn Material,
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
                let distance = if started_inside {
                    hit_range.1
                } else {
                    hit_range.0
                };
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
                    started_inside,
                    local_to_world: Mat3A::IDENTITY,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_bounds(&self) -> Option<(Vec3A, Vec3A)> {
        Some((
            self.center - Vec3A::splat(self.radius),
            self.center - Vec3A::splat(self.radius),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Plane<'a> {
    pub normal: Vec3A,
    pub center: Vec3A,
    pub material: &'a dyn Material,
    n_dot_c: f32,
    right: Vec3A,
    up: Vec3A,
}

impl<'a> Plane<'a> {
    pub fn new(
        normal: Vec3A,
        right: Vec3A,
        center: Vec3A,
        material: &'a dyn Material,
    ) -> Plane<'a> {
        let up = normal.cross(right);
        let right = up.cross(normal).normalize();
        let up = up.normalize();
        let normal = normal.normalize();
        Plane {
            normal,
            center,
            right,
            up,
            n_dot_c: normal.dot(center),
            material,
        }
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
                    world_normal: if n_dot_o >= 0. {
                        self.normal
                    } else {
                        -self.normal
                    },
                    local_pos: Vec3A::new(
                        self.right.dot(local_offset),
                        self.up.dot(local_offset),
                        0.,
                    ),
                    local_normal: if n_dot_o >= 0.0 {
                        Vec3A::Z
                    } else {
                        Vec3A::NEG_Z
                    },
                    material: self.material,
                    distance: dist,
                    started_inside: false,
                    local_to_world: Mat3A {
                        x_axis: self.right,
                        y_axis: self.right,
                        z_axis: self.normal,
                    },
                });
            }
        }
        None
    }

    fn get_bounds(&self) -> Option<(Vec3A, Vec3A)> {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cuboid<'a> {
    // You might have called this a "Box", except the word "Box" is already kind of a big deal in Rust.
    world_to_local: Affine3A,
    local_to_world: Affine3A,
    mins: Vec3A,
    maxs: Vec3A,
    material: &'a dyn Material,
}

impl<'a> Cuboid<'a> {
  pub fn new(origin: Vec3A, orient: Quat, mins: Vec3A, maxs: Vec3A, material: &'a dyn Material) -> Cuboid {
    let local_to_world = Affine3A::from_rotation_translation(orient, origin.into());
    Cuboid {
      world_to_local: local_to_world.inverse(), 
      local_to_world,
      mins: mins.min(maxs),
      maxs: mins.max(maxs),
      material
    }
  }
}

impl<'a> Shape for Cuboid<'a> {
    fn trace_ray(&self, ray: Ray) -> Option<Hit> {
        let local_origin = self.world_to_local.transform_point3a(ray.origin);
        let local_dir = self.world_to_local.transform_vector3a(ray.direction);
        let a = (self.mins - local_origin) / local_dir;
        let b = (self.maxs - local_origin) / local_dir;
        let near_dist = a.min(b).max_element();
        let far_dist = a.max(b).min_element();
        if near_dist <= far_dist && far_dist > 0. {
            let started_inside = near_dist <= 0.;
            let dist = if started_inside { far_dist } else { near_dist };
            let local_pos = local_origin + dist * local_dir;
            let mins_dist = (self.mins - local_pos).abs();
            let maxs_dist = (self.maxs - local_pos).abs();
            let mut best = mins_dist.x;
            let mut local_norm = Vec3A::NEG_X;
            let mut world_norm = -self.local_to_world.matrix3.x_axis;

            if mins_dist.y < best {
                best = mins_dist.y;
                local_norm = Vec3A::NEG_Y;
                world_norm = -self.local_to_world.matrix3.y_axis;
            }

            if mins_dist.z < best {
                best = mins_dist.z;
                local_norm = Vec3A::NEG_Z;
                world_norm = -self.local_to_world.matrix3.z_axis;
            }

            if maxs_dist.x < best {
                best = maxs_dist.x;
                local_norm = Vec3A::X;
                world_norm = self.local_to_world.matrix3.x_axis;
            }

            if maxs_dist.y < best {
                best = maxs_dist.y;
                local_norm = Vec3A::Y;
                world_norm = self.local_to_world.matrix3.y_axis;
            }

            if maxs_dist.z < best {
                //best = maxs_dist.z;
                local_norm = Vec3A::Z;
                world_norm = self.local_to_world.matrix3.z_axis;
            }

            Some(Hit {
                world_pos: self.local_to_world.transform_point3a(local_pos),
                world_normal: world_norm,
                local_pos: local_pos,
                local_normal: local_norm,
                material: self.material,
                distance: dist,
                started_inside: started_inside,
                local_to_world: self.local_to_world.matrix3,
            })
        } else {
            None
        }
    }

    fn get_bounds(&self) -> Option<(Vec3A, Vec3A)> {
        let r = 0.5 * (self.maxs - self.mins);
        let world_center = self.local_to_world.transform_point3a(self.mins + r);
        let world_r = (self.local_to_world.matrix3.x_axis * r.xxx()).abs()
            + (self.local_to_world.matrix3.y_axis * r.yyy()).abs()
            + (self.local_to_world.matrix3.z_axis * r.zzz()).abs();
        Some((world_center - world_r, world_center + world_r))
    }
}
