pub fn convert_to_arrow(c: char) -> Option<String> {
    if let Some(i) = c.to_digit(10) {
        match i {
            1 => Some("default\\down-left.gif".to_string()),
            2 => Some("default\\down.gif".to_string()),
            3 => Some("default\\down-right.gif".to_string()),
            4 => Some("default\\left.gif".to_string()),
            5 => Some("default\\neutral.png".to_string()),
            6 => Some("default\\right.gif".to_string()),
            7 => Some("default\\up-left.gif".to_string()),
            8 => Some("default\\up.gif".to_string()),
            9 => Some("default\\up-right.gif".to_string()),
            _ => Some("default\\err.png".to_string()),
        }
    } else {
        None
    }
}
