/// Returns "Yes" if input is true and "No" if input is false
pub fn yes_or_no(input: bool) -> String {
    if input {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

/// Converts the given frequency into a channel number
#[allow(unused)]
pub fn get_channel(frequency: u32) -> u32 {
    if (2412..=2472).contains(&frequency) {
        (frequency - 2407) / 5
    } else if frequency == 2484 {
        14 // japan
    } else if (5180..=5895).contains(&frequency) {
        (frequency - 5000) / 5
    } else if (5955..=7115).contains(&frequency) {
        (frequency - 5950) / 5
    } else {
        0
    }
}
