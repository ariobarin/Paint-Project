// Python's random module (the Mersenne Twister and randint), so a seed sprays the same dots as the
// Python version

pub struct Random {
    mt: [u32; 624],
    index: usize,
}

impl Random {
    fn init_genrand(s: u32) -> Random {
        let mut mt = [0u32; 624];
        mt[0] = s;
        for i in 1..624 {
            mt[i] = 1812433253u32.wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_add(i as u32);
        }
        Random { mt, index: 624 }
    }

    // random.seed(n) for a non-negative integer n
    pub fn new(seed: u64) -> Random {
        let key: Vec<u32> = if seed >> 32 != 0 { vec![seed as u32, (seed >> 32) as u32] } else { vec![seed as u32] };
        let mut r = Random::init_genrand(19650218);
        let mt = &mut r.mt;
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..624.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1664525)).wrapping_add(key[j]).wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= 624 {
                mt[0] = mt[623];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..623 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1566083941)).wrapping_sub(i as u32);
            i += 1;
            if i >= 624 {
                mt[0] = mt[623];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        r
    }

    fn genrand_u32(&mut self) -> u32 {
        if self.index >= 624 {
            for k in 0..624 {
                let y = (self.mt[k] & 0x8000_0000) | (self.mt[(k + 1) % 624] & 0x7fff_ffff);
                self.mt[k] = self.mt[(k + 397) % 624] ^ (y >> 1) ^ if y & 1 != 0 { 0x9908_b0df } else { 0 };
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    // random.randint(a, b): a + _randbelow(b - a + 1), rejection sampling on getrandbits
    pub fn randint(&mut self, a: i32, b: i32) -> i32 {
        let n = (b - a + 1) as u32;
        let k = 32 - n.leading_zeros();
        loop {
            let r = self.genrand_u32() >> (32 - k);
            if r < n {
                return a + r as i32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_python() {
        // random.seed(42); [random.randint(-10, 10) for _ in range(5)] -> [10, -7, -10, -2, -3]
        let mut r = Random::new(42);
        let got: Vec<i32> = (0..5).map(|_| r.randint(-10, 10)).collect();
        assert_eq!(got, [10, -7, -10, -2, -3]);
    }
}
