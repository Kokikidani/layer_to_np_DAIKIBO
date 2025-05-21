use rand::{ Rng, SeedableRng };
use rand_chacha::ChaCha8Rng;

pub fn enumerate_subsequences<T: Clone>(
    arr: &[T],
    min_length: usize,
    max_length: Option<usize>
) -> Vec<Vec<T>> {
    let mut subsequences = Vec::new();
    let n = arr.len();

    for start in 0..n {
        for end in start + min_length..=n {
            if let Some(max_len) = max_length {
                if end - start > max_len {
                    break;
                }
            }
            subsequences.push(arr[start..end].to_vec());
        }
    }

    subsequences
}

pub struct Arange {
    #[allow(dead_code)]
    start: f64,
    stop: f64,
    step: f64,
    current: f64,
}

impl Arange {
    fn new(start: f64, stop: f64, step: f64) -> Self {
        Arange {
            start,
            stop,
            step,
            current: start,
        }
    }
}

impl Iterator for Arange {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.stop {
            let result = self.current;
            self.current += self.step;
            Some(result)
        } else {
            None
        }
    }
}

pub fn arange(start: f64, stop: f64, step: f64) -> Arange {
    Arange::new(start, stop, step)
}

pub fn shuffle_array<T>(arr: &mut [T], rand_seed: u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(rand_seed);
    let mut n = arr.len();
    while n > 1 {
        let k = rng.gen_range(0..n);
        n -= 1;
        arr.swap(n, k);
    }
}
