use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand::seq::SliceRandom;

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
            r_vectors[i] = (rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));
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
        // corners
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

        // distance vectors from corners to p
        let sa = (u         , v         );
        let sb = (u         , v - 1.0   );
        let sc = (u - 1.0   , v - 1.0   );
        let sd = (u - 1.0   , v         );

        // dot products of gradient vectors and distance vectors
        let da = a.0 * sa.0 + a.1 * sa.1;
        let db = b.0 * sb.0 + b.1 * sb.1;
        let dc = c.0 * sc.0 + c.1 * sc.1;
        let dd = d.0 * sd.0 + d.1 * sd.1;

        // lerping between corners to p
        let l1 = da + (db - da) * u;
        let l2 = dc + (dd - dc) * u;
        let n = l1 + (l2 - l1) * v;

        // ease n and return
        // easing function here is 6n^5 - 15n^4 + 10n^3 (magic)
        n * n * n * (n * (n * 6.0 - 15.0) + 10.0)
    }

    fn grad(&self, x: u8, y: u8) -> (f32, f32) {
        let p = self.r_permutation;
        let hashed = p[(p[x as usize] as usize) + y as usize];
        self.r_vectors[hashed as usize]
    }
}
