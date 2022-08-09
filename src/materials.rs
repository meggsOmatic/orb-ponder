use crate::geom::*;
use crate::scene::*;
use crate::shapes::*;

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

pub trait Material: std::fmt::Debug + dyn_clone::DynClone + Sync {
    fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A;
}

#[derive(Debug, Copy, Clone)]
pub struct Lambertian(pub Vec3A);

impl Material for Lambertian {
    fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A {
        if !ctx.try_push() {
            return Vec3A::ZERO;
        }

        let color = self.0
            * scene.get_color(
                Ray {
                    origin: hit.world_pos,
                    direction: (hit.world_normal + 0.999 * get_point_in_sphere(ctx)).normalize(),
                },
                ctx,
            );
        ctx.pop();
        color
    }
}

#[derive(Debug, Copy, Clone)]

pub struct GlossWrap {
    pub gloss_color: Vec3A,
    pub diffuse_color: Vec3A,
    pub fresnel_power: f32,
    pub gloss_size: f32,
    pub max_gloss: f32,
    pub min_gloss: f32,
}

impl Material for GlossWrap {
    fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A {
        if !ctx.try_push() {
            return Vec3A::ZERO;
        }

        let fresnel = (1.0 + ray.direction.dot(hit.world_normal))
            .clamp(0., 1.)
            .powf(self.fresnel_power)
            .lerp(self.max_gloss, self.min_gloss);

        let gloss_dir = reflect(ray.direction, ctx.blur_vector(hit.world_normal, self.gloss_size));
        let diffuse_dir = ctx.blur_vector(hit.world_normal, 1.0);
        let color = if ctx.rng1() >= fresnel {
            self.diffuse_color
                * scene.get_color(
                    Ray {
                        origin: hit.world_pos,
                        direction: diffuse_dir,
                    },
                    ctx,
                )
        } else if gloss_dir.dot(hit.world_normal) > 0. {
            self.gloss_color
                * scene.get_color(
                    Ray {
                        origin: hit.world_pos,
                        direction: gloss_dir,
                    },
                    ctx,
                )
        } else {
            Vec3A::ZERO
        };
        ctx.pop();
        color
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Checkerboard<'a> {
    pub size: f32,
    pub a: &'a dyn Material,
    pub b: &'a dyn Material,
}

impl<'a> Material for Checkerboard<'a> {
    fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A {
        let c = (hit.local_pos * Vec3A::splat(1. / self.size)).floor();
        if (c.x as i32 ^ c.y as i32 ^ c.z as i32) & 1 == 0 {
            self.a.get_color(scene, ray, hit, ctx)
        } else {
            self.b.get_color(scene, ray, hit, ctx)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BrushedMetal {
    pub size: f32,
    pub radial_roughness: f32,
    pub circumference_roughness: f32,
    pub color: Vec3A
}

impl Material for BrushedMetal {
    fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A {
        if !ctx.try_push() {
            return Vec3A::ZERO;
        }
        let local = hit.local_pos * Vec3A::splat(1. / self.size);
        let center = local.floor() + Vec3A::splat(0.5);
        let offset = local - center;
        let radial_dir = offset - offset.dot(hit.local_normal) * hit.local_normal;
        let circumference_dir = hit.local_normal.cross(radial_dir);
        let sphere = get_point_in_sphere(ctx);
        let local_normal = radial_dir * (sphere.x * self.radial_roughness / radial_dir.length())
            + circumference_dir * (sphere.y * self.circumference_roughness / radial_dir.length())
            + hit.local_normal * (sphere.z * 0.999);
        let world_normal = (hit.local_to_world * local_normal).normalize();
        let reflected = reflect(ray.direction, world_normal);
        let color = if reflected.dot(hit.world_normal) > 0. {
          self.color * scene.get_color(Ray { origin: hit.world_pos, direction: reflected }, ctx)
        } else {
          Vec3A::ZERO
        };

        ctx.pop();
        color
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Emitter {
  pub color: Vec3A,
  pub focus: f32,
}


impl Material for Emitter {
  fn get_color(&self, scene: &Scene, ray: Ray, hit: &Hit, ctx: &mut TraceContext) -> Vec3A {
    self.color * (-ray.direction.dot(hit.world_normal)).clamp(0.00001, 1.0).powf(self.focus)
  }
}

/*
fn get_color(r: Ray, max_depth: i32, rng: &mut ThreadRng) -> Vec3A {
    let sphere = Sphere { center: Vec3A::new(0., 0., 1.5), radius: 1.5 };
    let sphere_hit = sphere.intersect(r);
    let dist_to_sphere = if let Some((near, _)) = sphere_hit { near } else { -1. };
    let dist_to_floor = r.origin.z / -r.direction.z;
    if dist_to_sphere > 0.01 && (dist_to_sphere < dist_to_floor || dist_to_floor <= 0.01) {
        if max_depth <= 0 { return Vec3A::ZERO; }
        let hit_pos = r.at(dist_to_sphere);
        let norm = (hit_pos - sphere.center).normalize();
        let spec_dir = (r.direction - 2. * norm.dot(r.direction) * norm + 0.05 * get_point_in_sphere(rng)).normalize();
        let diffuse_dir = (norm * 1.01 + get_point_in_sphere(rng)).normalize();
        let fresnel = (1.0 + r.direction.dot(norm)).powf(2.0).lerp(1.0, 0.1);
        let diffuse_color = Vec3A::new(0.35, 0.4, 0.5) * get_color(Ray { origin: hit_pos, direction: diffuse_dir }, max_depth - 1, rng);
        let spec_color = Vec3A::splat(1.) * get_color(Ray { origin: hit_pos, direction: spec_dir }, max_depth - 1, rng);
        diffuse_color.lerp(spec_color, fresnel)
    } else if dist_to_floor > 0.01 {
        let hit = r.at(dist_to_floor);
        if (hit.x.floor() as i32 ^ hit.y.floor() as i32) & 1 != 0 {
            Vec3A::new(0.4, 0.3, 0.1) * get_color(Ray { origin: hit, direction: (Vec3A::new(0., 0., 1.01) + get_point_in_sphere(rng)).normalize() }, max_depth - 1, rng)
        } else {
            let mut direction = r.direction;
            direction.z *= -1.0;
            direction += 0.1 * get_point_in_sphere(rng);
            direction = direction.normalize();
            1.5 * Vec3A::new(0.2, 0.15, 0.1) * get_color(Ray { origin: hit, direction: direction }, max_depth - 1, rng)
        }
    } else {
        1.0 * Vec3A::new(0.1, 0.2, 0.3) + 0.04 * r.direction.dot(Vec3A::new(0.8, 1.2, 1.6).normalize()).max(0.).powf(10.) * Vec3A::new(200., 175., 150.)
    }
}
*/
