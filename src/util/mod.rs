use rand::Rng;

pub fn generate_random_u32() -> u32 {
    let mut rng = rand::rng();
    rng.random::<u32>()
}