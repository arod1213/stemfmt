use regex::Regex;

pub fn get_regex(vals: &Vec<&str>) -> Regex {
    let super_pattern = format!(r"(?i)({})", vals.join("|"));
    Regex::new(&super_pattern).unwrap()
}

pub fn strip_name(name: &mut String) {
    *name = name
        .chars()
        .map(|c| {
            if !c.is_whitespace() && !c.is_digit(10) && !c.is_alphabetic() {
                return ' ';
            };
            c
        })
        .collect();
}

mod test {
    #[cfg(test)]
    use super::*;

    #[test]
    fn strip() {
        let mut name = "aidan --- tests this".to_string();
        strip_name(&mut name);
        assert_eq!(name, "aidan     tests this".to_string());

        let mut name = "a123".to_string();
        strip_name(&mut name);
        assert_eq!(name, "a123".to_string());
    }
}
