pub fn round_up(value: usize, place: usize) -> usize {
    assert!(place.is_power_of_two());
    round_down(value + place - 1, place)
}

pub fn round_down(value: usize, place: usize) -> usize {
    assert!(place.is_power_of_two());
    value & !(place - 1)
}
