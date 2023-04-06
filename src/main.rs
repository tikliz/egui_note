#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::{ColorImage, Label};
use egui_extras::RetainedImage;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

pub mod games;
use crate::games::logic_default;
use crate::games::logic_skg;
use crate::games::logic_skg::translate_inputs;

const GAME_LIST: &str = "src\\games\\game_list.json";

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    // let input_num_direc = HashMap::from([
    //     (1, "..\\images\\arrows\\1.gif".to_string()),
    //     (2, "..\\images\\arrows\\2.gif".to_string()),
    //     (3, "..\\images\\arrows\\3.gif".to_string()),
    //     (4, "..\\images\\arrows\\4.gif".to_string()),
    //     (5, "⊡".to_string()),
    //     (6, "..\\images\\arrows\\6.gif".to_string()),
    //     (7, "..\\images\\arrows\\7.gif".to_string()),
    //     (8, "..\\images\\arrows\\8.gif".to_string()),
    //     (9, "..\\images\\arrows\\9.gif".to_string()),
    // ]);

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(340.0, 460.0)),
        always_on_top: true,
        drag_and_drop_support: true,
        resizable: true,
        ..Default::default()
    };
    eframe::run_native(
        "Combo List",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

// "0": {
//     "name": "test 1",
//     "inputs": "236P LK LP\n2P 5MK",
//     "state": "n_done"

// },

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Character {
    name: String,
    combos: Vec<Option<Combo>>,
}
impl Character {
    fn new(name: String, combos: Vec<Option<Combo>>) -> Self {
        Self { name, combos }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Combo {
    name: String,
    inputs: String,
    state: ComboState,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ComboState {
    NotDone,
    Done,
    Testing,
}

struct MyApp {
    inputs: String,
    game_name: String,
    render_input: bool,
    new_line: bool,
    read_game_list: bool,
    game_list: Option<Value>,
    game_selected: Option<String>,
    read_character_list: bool,
    character_list: Option<Vec<Character>>,
    character_selected: Option<Character>,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut vec: Vec<Character> = Vec::new();
        Self {
            inputs: "2LK(1) 2MP 5HK
jMK jHP
5HP 236LP+LK
2MP 5[HP] 214P~P
5LK 5MK 2HP 236LP
5LP 2LK 5MP 5HP 236LP+LK
LP
HK
1"
            .to_owned(),
            game_name: "".to_owned(),
            new_line: false,
            render_input: false,
            read_game_list: true,
            game_list: None,
            game_selected: None,
            read_character_list: true,
            character_list: Some(vec),
            character_selected: None,
        }
    }
}

trait RemoveQuotes {
    fn remove_quotes(self) -> Self;
}

impl RemoveQuotes for String {
    fn remove_quotes(self) -> Self {
        self.replace('\"', "")
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // println!("update!");

        fn get_character_list(nself: &mut MyApp) {
            nself.read_character_list = false;
            nself.character_selected = None;
            if let Some(selected) = nself
                .game_list
                .to_owned()
                .unwrap()
                .get(nself.game_selected.to_owned().unwrap())
            {
                nself.character_list.as_mut().unwrap().clear();
                let selected = selected.to_string().remove_quotes();
                let character_list_str =
                    fs::read_to_string(Path::new(&format!("src\\games\\input_{}.json", selected)))
                        .expect("unable to read input_().json");
                let json: Value =
                    serde_json::from_str(&character_list_str).expect("bad input_().json");
                let characters = json.get("characters").unwrap();
                match characters {
                    Value::Object(obj) => {
                        for (k, v) in obj.iter() {
                            let temp: Value = serde_json::from_value(v.clone()).unwrap();
                            let temp: Value = temp.get("combos").unwrap().to_owned();
                            let mut temp_combos: Vec<Option<Combo>> = Vec::new();
                            match temp {
                                Value::Object(combo_obj) => {
                                    for (combo_k, combo_v) in combo_obj {
                                        let combo: Option<Combo> =
                                            serde_json::from_value(combo_v).unwrap();
                                        temp_combos.push(combo);
                                    }
                                }
                                _ => panic!(),
                            }
                            // println!("{} - {:?}\n", k, v);
                            let temp_character: Character =
                                Character::new(k.to_string(), temp_combos);
                            nself.character_list.as_mut().unwrap().push(temp_character);
                        }
                    }
                    _ => {
                        panic!("invalid json");
                    }
                };
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::CollapsingHeader::new("GAME OPTIONS")
                .default_open(true)
                .show(ui, |ui| {
                    if self.read_game_list {
                        self.read_game_list = false;
                        let games_list_str = fs::read_to_string(Path::new(GAME_LIST))
                            .expect("Unable to read game_list_name.json");
                        self.game_list = serde_json::from_str(&games_list_str).expect("bad json.");
                        if let Some(selected) =
                            self.game_list.to_owned().unwrap().get("previous_choice")
                        {
                            if selected != "null" {
                                self.game_selected = Some(selected.to_string().remove_quotes());
                            };
                        }
                        get_character_list(self);
                    }

                    // ui.label("Contents");
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let temp_selection = self.game_selected.clone();

                            egui::ComboBox::from_label("")
                                .selected_text(
                                    self.game_selected
                                        .to_owned()
                                        .unwrap_or("Select a game.".to_string()),
                                )
                                .show_ui(ui, |ui| {
                                    match self.game_list.to_owned().unwrap() {
                                        Value::Object(obj) => {
                                            for (k, v) in obj.iter() {
                                                if k != "previous_choice" {
                                                    ui.selectable_value(
                                                        &mut self.game_selected,
                                                        Some(k.clone()),
                                                        k,
                                                    );
                                                }
                                            }
                                        }
                                        _ => {
                                            panic!("json is not an object.");
                                        }
                                    }
                                    if self.character_list.is_some() {
                                        self.read_character_list =
                                            self.game_selected != temp_selection;
                                    }
                                    if self.read_character_list {
                                        // save game choice to json
                                        if let Some(choosen_game) = self
                                            .game_list
                                            .as_mut()
                                            .unwrap()
                                            .get_mut("previous_choice")
                                        {
                                            *choosen_game = Value::String(
                                                self.game_selected.to_owned().unwrap(),
                                            );
                                        }
                                        std::fs::write(
                                            Path::new(GAME_LIST),
                                            serde_json::to_string_pretty(&self.game_list).unwrap(),
                                        )
                                        .unwrap();
                                        get_character_list(self);
                                    }
                                });

                            if self.game_selected.is_some() {
                                if let Some(selected) = self
                                    .game_list
                                    .to_owned()
                                    .unwrap()
                                    .get(self.game_selected.to_owned().unwrap())
                                {
                                    ui.end_row();
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::from_label("")
                                            .selected_text(
                                                if self.character_selected.is_some() {
                                                    self.character_selected.as_ref().unwrap().name.clone()

                                                } else {
                                                    "Select a character".to_string()

                                                }
                                            )
                                            .show_ui(ui, |ui| {
                                                if self.character_list.is_none() {
                                                    get_character_list(self);
                                                } else {
                                                    for c in self.character_list.as_ref().unwrap() {
                                                        let c_name =
                                                            c.name.to_owned().remove_quotes();
                                                        ui.selectable_value(
                                                            &mut self.character_selected,
                                                            Some(c.to_owned()),
                                                            c_name,
                                                        );
                                                    }
                                                }
                                            });
                                    });
                                }
                            }
                        });

                        //ui.add_space(70.00);
                        egui::CollapsingHeader::new("ADD").show(ui, |ui| {
                            let game_name = ui.label("Body");
                            ui.text_edit_singleline(&mut self.game_name)
                                .labelled_by(game_name.id);
                        });
                    });
                });
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::CollapsingHeader::new("INPUTS").show(ui, |ui| {
                    let name_label = ui.label("Inputs: ");
                    ui.text_edit_multiline(&mut self.inputs)
                        .labelled_by(name_label.id);
                });
                // ui.horizontal(|ui| {
                //     let name_label = ui.label("Inputs: ");
                //     ui.text_edit_multiline(&mut self.inputs)
                //         .labelled_by(name_label.id);
                // });
                // ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if self.render_input && self.inputs.len() > 0 {
                    ui.horizontal_wrapped(|ui| {
                        //ui.label("test");
                        let mut temp_char: Option<String> = None;
                        let mut append_next = false;
                        let mut weight = None;
                        for c in self.inputs.chars() {
                            temp_char = logic_default::convert_to_arrow(c);
                            if temp_char.is_none() {
                                temp_char = translate_inputs(
                                    &mut append_next,
                                    &mut weight,
                                    c,
                                    &mut self.new_line,
                                );
                            };
                            if temp_char.is_some() {
                                if self.new_line {
                                    self.new_line = false;
                                    ui.end_row();
                                    ui.vertical(|ui| {
                                        ui.separator();
                                    });
                                } else {
                                    let temp_char = temp_char.unwrap().to_string();
                                    let mut images_path = String::new();
                                    if let Some(selected) = self
                                        .game_list
                                        .to_owned()
                                        .unwrap()
                                        .get(self.game_selected.to_owned().unwrap())
                                    {
                                        if !temp_char.contains("default") {
                                            let selected = selected.to_string().remove_quotes();
                                            images_path =
                                                format!("images\\{}\\{}", selected, temp_char);
                                        } else {
                                            images_path = format!("images\\{}", temp_char);
                                        }
                                    }
                                    // let selected = self.game_list.clone().unwrap().get(self.game_selected.clone().unwrap());
                                    // println!("{}", temp_char);
                                    // let tmp = format!("images\\{}\\{}", selected.unwrap(), temp_char);
                                    let mut temp_bytes = File::open(images_path).unwrap();
                                    let mut buffer = Vec::new();
                                    temp_bytes.read_to_end(&mut buffer).unwrap();

                                    let image = RetainedImage::from_image_bytes(temp_char, &buffer)
                                        .unwrap();
                                    image.show(ui);
                                }
                            };

                            // // Show the image:
                            // ui.add(egui::Image::new(texture, texture.size_vec2()));

                            // // Shorter version:
                            // ui.image(texture, texture.size_vec2());
                        }
                    });
                }
                if ui.button("Toggle").clicked() {
                    self.render_input = !self.render_input;
                }

                //ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });
        });
    }
}
