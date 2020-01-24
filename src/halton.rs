
pub struct Halton {
    b:  u32,
    rb: f32,
    i:  u32,
    d:  u32,
}

impl Halton {
    //pub fn new(b: u32, rng: &mut Rand) -> Halton {
    //    Self::new_seed(b, rng.gen())
    //}

    pub fn new_seed(b: u32, i: u32) -> Halton {
        Self::new_d(b, i, 1)
    }

    pub fn new_d(b: u32, i: u32, d: u32) -> Halton {
        Halton { b, rb: 1. / b as f32, i, d }
    }

    pub fn next(&mut self) -> f32 {
        let mut f = 1.;
        let mut r = 0.;
        let mut n = self.i;
        while n > 0 {
            f *= self.rb;
            r += f * (n % self.b) as f32;
            n /= self.b;
        }
        self.i += self.d;
        r
    }
}

