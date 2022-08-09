#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod materials;
mod shapes;
mod geom;
mod scene;

use crate::materials::*;
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
use clap::{Parser, Subcommand};
use rayon::{ *, iter::* };

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    samples: Option<u32>,

    #[clap(short, long, value_parser)]
    maxdepth: Option<u32>,

    #[clap(short, long, value_parser)]
    width: Option<u32>,

    #[clap(short, long, value_parser)]
    height: Option<u32>,
}


#[derive(Clone, Copy, Debug)]
struct Viewport {
    width: f32,
    height: f32,
    v_fov: f32
}


impl Viewport {
    fn pixel_to_dir(&self, pixel: Vec2) -> Vec3A {
        let tan_half_v_fov = (0.5 * self.v_fov).tan();
        let tan_half_h_fov = (self.width * tan_half_v_fov) / self.height;
        let centered_pixel = (pixel + Vec2::splat(0.5)) - 0.5 * Vec2::new(self.width, self.height);
        let norm_pixel = centered_pixel / Vec2::new(self.width, self.height);
        let fov_pixel = norm_pixel * Vec2::new(2.0 * tan_half_h_fov, -2.0 * tan_half_v_fov);
        let fwd = Vec3A::from((fov_pixel, 1.0));
        let fwd_norm = fwd.normalize();
        fwd_norm
    }
}

// fn get_point_in_sphere(rng: &mut ThreadRng) -> Vec3A {
//     loop {
//         let r = Vec3A::new(rng.gen(), rng.gen(), rng.gen()) * Vec3A::splat(2.0) - Vec3A::ONE;
//         if r.length_squared() <= 1.0 { return r; }
//     }
// }




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

fn main() {
    let cli = Cli::parse();
    let width = cli.width.or(cli.height).unwrap_or(512);
    let height = cli.height.or(cli.width).unwrap_or(512);
    let mut dest = Rgb32FImage::new(width, height);

    let scene_to_eye = Affine3A::look_at_lh(Vec3::new(-10., -2., 2.75), Vec3::new(0., 1., 1.), Vec3::Z);
    let eye_to_scene = scene_to_eye.inverse();
    let viewport = Viewport { width: width as f32, height: height as f32, v_fov: 30_f32.to_radians() };


    let grey = Lambertian(Vec3A::new(0.5, 0.5, 0.5));
    let yellow = Lambertian(Vec3A::new(0.4, 0.3, 0.1));
    let gloss_floor = GlossWrap {
        gloss_color: Vec3A::new(0.3, 0.225, 0.15),
        diffuse_color: Vec3A::new(0., 0., 0.),
        gloss_size: 0.06,
        max_gloss: 1.0,
        min_gloss: 1.0,
        fresnel_power: 0.0
    };
    let yellow_floor = GlossWrap {
        gloss_color: Vec3A::ONE,
        diffuse_color: Vec3A::new(0.4, 0.3, 0.1),
        gloss_size: 0.2,
        max_gloss: 0.2,
        min_gloss: 0.0,
        fresnel_power: 5.0
    };
    let brushed_metal = BrushedMetal {
        size: 1.0,
        radial_roughness: 0.0,
        circumference_roughness: 0.15,
        color: Vec3A::new(0.8, 0.9, 1.0) * 0.2
    };
    let check = Checkerboard { size: 1.0, a: &gloss_floor, b: &yellow };



    let brushed_metal = BrushedMetal {
        size: 1.0,
        radial_roughness: 0.15,
        circumference_roughness: 0.0,
        color: Vec3A::new(0.8, 0.9, 1.0) * 0.4
    };
//    Lambertian(Vec3A::new(0.4, 0.3, 0.1)) };
    let sphere = GlossWrap {
        gloss_color: Vec3A::ONE,
        diffuse_color: Vec3A::new(0.35, 0.4, 0.5),
        gloss_size: 0.05,
        max_gloss: 1.0,
        min_gloss: 0.02,
        fresnel_power: 2.0
    };
    let red = Lambertian(Vec3A::new(1.0, 0.0, 0.0));
    let green_glow = Emitter { color: vec3a(0.4, 1.0, 0.4), focus: 1.0 };
    let white_glow = Emitter { color: vec3a(1.0, 1.0, 1.0) * 3., focus: 0.0 };

    let scene = Scene {
        shapes: vec![
            Box::new(Sphere {
                center: Vec3A::new(0., 0., 1.5),
                radius: 1.5,
                material: &sphere
            }),
            Box::new(Cuboid::new(vec3a(-2., 2.5, 0.), Quat::from_rotation_z(130.0f32.to_radians()), vec3a(-0.1, -2., 0.0), vec3a(0.1, 2., 4.0), &white_glow)),
            Box::new(Sphere {
                center: Vec3A::new(-1.25, -1.25, 0.5),
                radius: 0.5,
                material: &green_glow
            }),
            Box::new(Sphere {
                center: Vec3A::new(-1.25, 1.25, 0.5),
                radius: 0.5,
                material: &red
            }),
            Box::new(Plane::new(Vec3A::Z, Vec3A::X, Vec3A::ZERO, &check))
            ]
    };

    let bar = indicatif::ProgressBar::new((width * height) as u64);
    let num_aa = cli.samples.unwrap_or(10);
    let max_depth = cli.maxdepth.unwrap_or(5).max(1) as i32;
    let colors: Vec<_> = (0..(width * height)).into_par_iter().map(|pixel_number| {
        let x = pixel_number % width;
        let y = pixel_number / width;
        let mut total_color = Vec3A::ZERO;
        let mut trace_context = TraceContext::new(max_depth);
        for _ in 0..num_aa {
            let xy = Vec2::new(x as f32, y as f32) - Vec2::splat(0.5) + trace_context.rng2();
            let view_dir = viewport.pixel_to_dir(xy);
            let scene_ray = Ray {
                origin: eye_to_scene.translation,
                direction: eye_to_scene.transform_vector3a(view_dir)
            };
            let sample_color = scene.get_color(scene_ray, &mut trace_context);
            trace_context.next_sample();
            total_color += sample_color;
        }
        bar.inc(1);
        total_color / num_aa as f32
    }).collect();
    bar.finish();


    for (c, p) in colors.iter().zip(dest.pixels_mut()) {
        let srgb = |c: f32| {
            if c > 0.0 {
                if c <= 0.0031308 { c * 12.92 }
                else if c < 1.0 { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
                else { 1.0 }
            } else { 0.0 }
        };
        p[0] = srgb(c.x);
        p[1] = srgb(c.y);
        p[2] = srgb(c.z);
    }

    if false {
        for p in dest.pixels_mut() {
        p.apply(|c|
            if c > 0.0 {
                if c <= 0.04045 { c / 12.92 }
                else if c < 1.0 { ((c + 0.055) / 1.055).powf(2.4) }
                else { 1.0 }
            } else { 0.0 });
        //p.apply(|c| c.powf(2.2));
        }
    }

    if false {
        for p in dest.pixels_mut() {
        p.apply(|c|
            if c > 0.0 {
                if c <= 0.0031308 { c * 12.92 }
                else if c < 1.0 { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
                else { 1.0 }
            } else { 0.0 });
        //p.apply(|c| c.powf(2.2));
        }
    }


    let rgb888: RgbImage = dest.convert();
    rgb888.save("test.png").expect("Could not save image file");
}
