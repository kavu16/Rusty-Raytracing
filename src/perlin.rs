use crate::{
    utils::random_int,
    vec3::{Point3, Vec3},
};

#[derive(Clone)]
pub struct Perlin {
    randvec: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Perlin {
    pub fn new() -> Self {
        let mut randvec = Vec::with_capacity(256);

        for _ in 0..256 {
            randvec.push(Vec3::random_range(-1.0, 1.0).unit_vector());
        }

        Self {
            randvec,
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    pub fn noise(&self, p: Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c = [[[Vec3::default(); 2]; 2]; 2];

        for (di, x) in c.iter_mut().enumerate() {
            for (dj, y) in x.iter_mut().enumerate() {
                for (dk, z) in y.iter_mut().enumerate() {
                    *z = self.randvec[self.perm_x[((i + di as i32) & 255) as usize]
                    ^ self.perm_y[((j + dj as i32) & 255) as usize]
                    ^ self.perm_z[((k + dk as i32) & 255) as usize]];
                }
            }
        }

        Perlin::perlin_interp(c, u, v, w)
    }

    pub fn turb(&self, p: Point3, depth: i32) -> f64 {
        let mut temp_p = p;
        let mut weight = 1.0;

        (0..depth)
            .fold(0.0, |acc, _d| {
                let acc = acc + weight * self.noise(temp_p);
                weight *= 0.5;
                temp_p *= 2.0;
                acc
            })
            .abs()
    }

    fn perlin_generate_perm() -> Vec<usize> {
        let mut p = Vec::with_capacity(256);
        for i in 0..256 {
            p.push(i);
        }

        Perlin::permute(&mut p, 256);

        p
    }

    fn permute(p: &mut [usize], n: usize) {
        for i in (0..n).rev() {
            let target = random_int(0, i as i32) as usize;
            p.swap(i, target);
        }
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3. - 2. * u);
        let vv = v * v * (3. - 2. * v);
        let ww = w * w * (3. - 2. * w);
        let mut accum = 0.0;

        for (i, x) in c.iter().enumerate() {
            for (j, y) in x.iter().enumerate() {
                for (k, z) in y.iter().enumerate() {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1. - i as f64) * (1. - uu))
                        * (j as f64 * vv + (1. - j as f64) * (1. - vv))
                        * (k as f64 * ww + (1. - k as f64) * (1. - ww))
                        * z.dot(&weight_v);
                }
            }
        }

        accum
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for Perlin {}
unsafe impl Sync for Perlin {}
