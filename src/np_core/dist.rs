use rand::Rng;
use rand_chacha::ChaCha8Rng;

/// Generates a random interval following the Exponential Distribution.
///
/// This function employs the inversion method using uniform distribution pseudorandom numbers.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a ChaCha8Rng random number generator.
/// * `lambda` - The rate parameter, which should be set to the reciprocal of the traffic intensity.
///
/// # Returns
///
/// A random interval following the Exponential Distribution.
///
/// # Example
///
/// ```
/// use rand_chacha::ChaCha8Rng;
/// use rand::SeedableRng;
/// use layer_to_np2::np_core::dist::get_poisson_interval;
///
/// let mut rng = ChaCha8Rng::seed_from_u64(42);
/// let lambda = 0.5; // Adjust according to your use case
/// let interval = get_poisson_interval(&mut rng, lambda); println!("Generated interval: {}", interval);
/// ```
///
/// # References
///
/// * [Wikipedia: Exponential Distribution](https://en.wikipedia.org/wiki/Exponential_distribution#Random_variate_generation)
/// * [Wikipedia: 指数分布 (Japanese)](https://ja.wikipedia.org/wiki/指数分布#生成)
pub fn get_poisson_interval(rng: &mut ChaCha8Rng, lambda: f64) -> usize {
    // Generate a random value within the range of U(0,1)
    let u: f64 = rng.gen_range(0.0..1.0);

    // Calculate the interval
    (1.0 - u.ln() / lambda) as usize
}
