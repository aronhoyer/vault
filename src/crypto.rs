use rand::{thread_rng, Rng};

pub fn generate_password(length: usize) -> String {
    let charsets = [
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        "abcdefghijklmnopqrstuvwxyz",
        "0123456789",
        "!\"#$%&'()*+,-./\\:;?@[]^_`{|}~ ",
    ];

    let mut password = String::new();
    for _ in 0..length {
        let charset = charsets[thread_rng().gen_range(0..charsets.len())];
        password.push(
            charset
                .chars()
                .nth(thread_rng().gen_range(0..charset.len()))
                .unwrap(),
        );
    }
    return password;
}
