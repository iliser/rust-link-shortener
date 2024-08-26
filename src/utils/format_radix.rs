
pub fn format_radix(mut x: u128, radix: u32) -> String {
    let mut result = vec![];
    let radix = radix.min(36).max(2);

    loop {
        let m = x % radix as u128;
        x = x / radix as u128;

        result.push(std::char::from_digit(m as u32, radix).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}