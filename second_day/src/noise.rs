use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

use image::{DynamicImage, ImageBuffer, Rgb};

use cgmath::prelude::*;
use cgmath::Vector2;

pub struct PerlinNoise2DGenerator {
    seed: u64,
    r_vectors: [(f32, f32); 256],
    r_permutation: [u8; 256],
}

impl PerlinNoise2DGenerator {
    pub fn new(seed: u64) -> PerlinNoise2DGenerator {
        let mut rng: StdRng = StdRng::seed_from_u64(seed);

        let mut r_vectors = [(0.0, 0.0); 256];
        for i in 0..256 {
            // we generate the random vectors using polar coordinates, rather than just a random x and y,
            // because otherwise the gradients will unfairly favor diagonal-ish directions
            // (consider the parts of a square with radius 1 that go beyond the circle with radius 1)
            let angle = rng.gen_range(0.0, 1.0);
            let tau = std::f64::consts::PI as f32 * 2.0;
            let vector = Vector2::new(f32::cos(angle * tau), f32::sin(angle * tau));
            // normalizing is not necessary with this method, if an optimization is needed here
            let normalized_vector = vector.normalize();
            r_vectors[i] = (normalized_vector.x, normalized_vector.y);
        }

        // fill with 0 through 255, in order
        let mut r_permutation = [0; 256];
        for i in 0..256 {
            r_permutation[i] = i as u8;
        }
        r_permutation.shuffle(&mut rng);

        PerlinNoise2DGenerator {
            seed,
            r_vectors,
            r_permutation,
        }
    }

    pub fn at(&self, x: f32, y: f32) -> f32 {
        // corner coords
        let xf = x.floor() as u8;
        let xc = x.ceil() as u8;
        let yf = y.floor() as u8;
        let yc = y.ceil() as u8;
        // coordinates of p in cell space (rather than world space) e.g. 321.44 becomes 0.44
        let u = x % 1.0;
        let v = y % 1.0;

        // gradients at corners
        let a = self.grad(xf, yf);
        let b = self.grad(xf, yc);
        let c = self.grad(xc, yc);
        let d = self.grad(xc, yf);

        // difference vectors from corners to p
        let sa = (u, v);
        let sb = (u, v - 1.0);
        let sc = (u - 1.0, v - 1.0);
        let sd = (u - 1.0, v);

        // dot products of gradient vectors and distance vectors
        let da = a.0 * sa.0 + a.1 * sa.1;
        let db = b.0 * sb.0 + b.1 * sb.1;
        let dc = c.0 * sc.0 + c.1 * sc.1;
        let dd = d.0 * sd.0 + d.1 * sd.1;

        // weighted average of dots, weighted by distance of from their corner to p
        let l1 = da + (db - da) * u;
        let l2 = dc + (dd - dc) * u;
        let n = l1 + (l2 - l1) * v;

        // ease n and return
        // easing function here is 6n^5 - 15n^4 + 10n^3 (magic)
        let eased = n * n * n * (n * (n * 6.0 - 15.0) + 10.0);

        /*if eased <= 0.0 {
            println!(
            "Output for Point {}, {}
            Corner Coords: x{} x{} y{} y{}
            Cell-Space Coords: {} {}
            Grads at Corners: {:?} {:?} {:?} {:?}
            Difference Vectors from Corners: {:?} {:?} {:?} {:?}
            Dot Products of Grads and Dists: {} {} {} {}
            Lerped: {}
            Eased: {}",
            x, y,
            xf, xc, yf, yc,
            u, v,
            a, b, c, d,
            sa, sb, sc, sd,
            da, db, dc, db,
            n,
            eased
            );
        }*/

        eased
    }

    fn grad(&self, x: u8, y: u8) -> (f32, f32) {
        let p = self.r_permutation;
        // TODO there's no way these casts are good
        let hashed = p[(((p[x as usize] as usize) + y as usize) as u8) as usize];
        self.r_vectors[hashed as usize]
    }

    pub fn image(&self, x: u32, y: u32, width: u32, height: u32, scale: f32) -> DynamicImage {
        let mut img = ImageBuffer::new(width, height);
        for u in 0..width {
            for v in 0..height {
                // (a + 1) / 2, rather than just a, because a is from -1 to 1
                // (my implementation of perlin noise is incorrect and I'm leaving it)
                let n_float = (self.at((x + u) as f32 * scale, (y + v) as f32 * scale) + 1.0) / 2.0;
                let n_u8 = (n_float * 255.0).clamp(0.0, 255.0) as u8;
                let color = Rgb([n_u8, n_u8, n_u8]);
                img.put_pixel(u, v, color);
            }
        }
        DynamicImage::ImageRgb8(img)
    }
}
