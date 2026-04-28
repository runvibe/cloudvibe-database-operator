use rand::distr::{Alphanumeric, SampleString};

pub fn generate(length: usize) -> String {
    Alphanumeric.sample_string(&mut rand::rng(), length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_requested_length() {
        assert_eq!(generate(40).len(), 40);
    }
}
