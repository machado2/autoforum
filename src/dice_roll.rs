use rand::Rng;

pub fn dice_roll(chance_returning_true_in_100: i32) -> bool {
    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(1..101);
    roll <= chance_returning_true_in_100
}
