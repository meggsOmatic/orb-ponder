use std::io::Cursor;
use image::buffer::ConvertBuffer;
use image::io::Reader as ImageReader;
use image::*;
use glam::{ *, f32::* };
use quasirandom::Qrng;
use lerp::Lerp;
use rand::{Rng, rngs::ThreadRng, thread_rng};

#[derive(Clone, Copy, Debug)]
struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A
}

impl Ray {
    fn at(&self, dist: f32) -> Vec3A {
        self.origin + dist * self.direction
    }
}

#[derive(Clone, Copy, Debug)]
struct Viewport {
    width: f32,
    height: f32,
    vFOV: f32
}


impl Viewport {
    fn pixel_to_dir(&self, pixel: Vec2) -> Vec3A {
        let tan_half_v_fov = (0.5 * self.vFOV).tan();
        let tan_half_h_fov = (self.width * tan_half_v_fov) / self.height;
        let centered_pixel = (pixel + Vec2::splat(0.5)) - 0.5 * Vec2::new(self.width, self.height);
        let norm_pixel = centered_pixel / Vec2::new(self.width, self.height);
        let fov_pixel = norm_pixel * Vec2::new(2.0 * tan_half_h_fov, -2.0 * tan_half_v_fov);
        let fwd = Vec3A::from((fov_pixel, 1.0));
        let fwd_norm = fwd.normalize();
        fwd_norm
    }
}


struct Sphere {
    pub center: Vec3A,
    pub radius: f32
}

impl Sphere {
    fn intersect(&self, r: Ray) -> Option<(f32, f32)> {
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

fn get_point_in_sphere(rng: &mut ThreadRng) -> Vec3A {
    loop {
        let r = Vec3A::new(rng.gen(), rng.gen(), rng.gen()) * Vec3A::splat(2.0) - Vec3A::ONE;
        if r.length_squared() <= 1.0 { return r; }
    }
}

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


fn main() {
    let mut dest = Rgb32FImage::new(512, 512);
    if false {
        for y in 0..dest.height() {
            for x in 0..dest.width() {
                dest.put_pixel(x, y, Rgb([x as f32 / 511f32, y as f32 / 511f32, 0.5f32]));
            }
        }
    }


    let scene_to_eye = Affine3A::look_at_lh(Vec3::new(-8.0, -0.8, 1.75), Vec3::new(0., 0., 1.), Vec3::Z);
    dbg!(scene_to_eye);
    let eye_to_scene = scene_to_eye.inverse();
    dbg!(eye_to_scene);
    let viewport = Viewport { width: dest.width() as f32, height: dest.height() as f32, vFOV: 45f32.to_radians() };
    dbg!(viewport);

    let mut qrng = Qrng::<(f32, f32)>::new(0.69);
    let mut rng = thread_rng();
    for (x, y, p) in dest.enumerate_pixels_mut() {
        let num_aa = 1000;
        let mut total_color = Rgb([0., 0., 0.]);
        for _ in 0..num_aa {
            let xy = Vec2::new(x as f32, y as f32) - Vec2::splat(0.5) + Vec2::from(qrng.gen());
            let view_dir = viewport.pixel_to_dir(xy);
            //dbg!(x, y, view_dir);
            let norm = Vec3A::splat(0.5) + 0.5 * view_dir;
            let scene_ray = Ray {
                origin: eye_to_scene.translation,
                direction: eye_to_scene.transform_vector3a(view_dir)
            };
            let sample_color = get_color(scene_ray, 10, &mut rng);
            total_color[0] += sample_color[0];
            total_color[1] += sample_color[1];
            total_color[2] += sample_color[2];
        }
        total_color[0] /= num_aa as f32;
        total_color[1] /= num_aa as f32;
        total_color[2] /= num_aa as f32;

        //dbg!(scene_ray);
        *p = total_color;
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

    if true {
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
