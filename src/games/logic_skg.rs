use crate::games::logic_default;
use egui::Ui;
use egui_extras::RetainedImage;

pub fn check_weight(append_next: &mut bool, weight: &Option<String>, c: char) -> Option<String> {
    let c = c.to_lowercase();
    if *append_next {
        *append_next = false;
        Some(format!("Sg_{}{}.png", weight.as_ref().unwrap(), c))
    } else {
        Some(format!("Sg_{}.png", c))
    }
}

pub fn translate_inputs(
    append_next: &mut bool,
    weight: &mut Option<String>,
    c: char,
    nl: &mut bool,
) -> Option<String> {
    let temp_input = match c.to_ascii_uppercase() {
        'L' | 'M' | 'H' => {
            *append_next = true;
            *weight = Some(c.clone().to_string().to_lowercase());
            None
        }
        'P' => check_weight(append_next, weight, c),
        'K' => check_weight(append_next, weight, c),
        'J' => Some("default\\up.gif".to_string()),
        '+' => Some("default\\plus.png".to_string()),
        '~' => Some("default\\plus.png".to_string()),

        '\n' => {
            *nl = true;
            Some("default\\space.png".to_string())
        }
        ' ' => Some("default\\space.png".to_string()),
        _ => Some("default\\err.png".to_string()),
    };
    temp_input
}
