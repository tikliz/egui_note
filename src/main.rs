#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::Pos2;
use egui::{ColorImage, Label};
use egui_extras::RetainedImage;
use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::{default, fs};

pub mod games;

const GAME_LIST: &str = "src\\games\\game_list.json";
const WIDTH: f32 = 340.0;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(WIDTH, 460.0)),
        initial_window_pos: Some(Pos2 { x: 140.0, y: 0.0 }),
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

impl Combo {
    fn new(name: String, inputs: String, state: ComboState) -> Self {
        Self {
            name,
            inputs,
            state,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ComboState {
    NotDone,
    Done,
    Testing,
}

struct MyApp {
    show_window: bool,
    new_inputs: String,
    inputs: String,
    combo_selector: f32,
    description: String,
    show_images: bool,
    new_line: bool,
    read_game_list: bool,
    game_list: Option<Value>,
    game_selected: Option<String>,
    get_images: bool,
    default_json: Value,
    retained_images: Vec<Option<RetainedImage>>,
    mapped_inputs: Vec<Option<Vec<String>>>,
    game_json: Option<Value>,
    game_path: Option<String>,
    changed_inputs: bool,
    read_character_list: bool,
    character_list: Option<Vec<Character>>,
    character_selected: Option<Character>,
    previous_choice: Option<String>,
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
            show_window: false,
            description: "".to_owned(),
            new_inputs: "".to_owned(),
            combo_selector: 0.0,

            get_images: true,
            retained_images: vec![None],
            mapped_inputs: vec![None],
            changed_inputs: true,
            new_line: false,
            show_images: false,
            read_game_list: true,
            game_list: None,
            game_selected: None,
            default_json: serde_json::from_str(include_str!("games\\input_default.json")).unwrap(),
            game_json: None,
            game_path: None,
            read_character_list: true,
            character_list: Some(vec),
            character_selected: None,
            previous_choice: None,
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
const REGEX_STUFF:  [&str; 7] = ["+", "*", "?", "[", "]", "(", ")"];
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // println!("update!");
        fn add_combo(c: Character, nself: &mut MyApp, new_combo: Combo) {
            nself
                .character_selected
                .as_mut()
                .unwrap()
                .combos
                .push(Some(new_combo.clone()));
            let mut i: usize;
            if c.combos.is_empty() {
                i = 0;
            } else {
                i = c.combos.len();
            }
            //let temp_json = serde_json::to_string(&test[2]).unwrap();
            let mut combos_map = IndexMap::new();
            for (i, combo) in c.combos.into_iter().enumerate() {
                combos_map.insert(i.to_string(), combo);
            }
            combos_map.insert(i.to_string(), Some(new_combo));
            if let Some(character) =
                nself.game_json.as_mut().unwrap()["characters"][c.name].as_object_mut()
            {
                if let Some(combos) = character["combos"].as_object_mut() {
                    // convert indexmap to string, then to a json_literal
                    let json_literal = serde_json::to_string(&combos_map).unwrap();
                    let json_literal: Value = serde_json::from_str(json_literal.as_str()).unwrap();
                    *combos = json_literal.as_object().unwrap().to_owned();
                }
            }
            let formatted_json =
                serde_json::to_string_pretty(nself.game_json.as_ref().unwrap()).unwrap();
            //println!("adding {}", formatted_json);
            fs::write(
                Path::new(&nself.game_path.to_owned().unwrap()),
                formatted_json,
            )
            .unwrap();
            let character_list_str =
                fs::read_to_string(Path::new(nself.game_path.as_ref().unwrap()))
                    .expect("unable to read input_().json");
            nself.game_json = serde_json::from_str(&character_list_str).expect("bad input_().json");
        }

        fn get_character_list(nself: &mut MyApp) {
            nself.combo_selector = 0.0;
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
                nself.game_path = Some(format!("src\\games\\input_{}.json", selected));
                let character_list_str =
                    fs::read_to_string(Path::new(nself.game_path.as_ref().unwrap()))
                        .expect("unable to read input_().json");
                nself.game_json =
                    serde_json::from_str(&character_list_str).expect("bad input_().json");
                let characters = nself.game_json.as_ref().unwrap().get("characters").unwrap();
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
                // println!("game list - {:?}", nself.game_list);
            }
        }
        fn compare_combinations(
            combinations: &mut String,
            attacks: &mut [String],
        ) -> Option<Vec<String>> {
            for a in attacks.iter_mut() {
                //println!("274: {}", a);
                if *a == "+" || *a == "*" || *a == "?"  || *a == "[" || *a == "]" {
                    *a = format!("\\{}", a);
                }

            }
            // *combinations = combinations.replace('(', "\\(");
            // *combinations = combinations.replace(')', "\\)");

            let attacks_re = attacks.join("|").to_ascii_uppercase();
            let re = Regex::new(&attacks_re).unwrap();
            // println!("285: {:?}", re);
            let mut temp_inputs: Vec<String> = Vec::new();
            let mut mapped_inputs: Vec<Vec<String>> = Vec::new();
            let mut temp_combination = combinations.to_string();
            while temp_inputs.join("").len() < combinations.len() {
                while let Some(m) = re.find(temp_combination.clone().as_str()) {
                    let temp_temp = temp_combination.clone();
                    let mut tokens: Vec<&str> = temp_temp.splitn(2, m.as_str()).collect();
                    // println!("293: {:?}", tokens);
                    if !tokens[0].is_empty() {
                        temp_combination.remove(0);
                        temp_inputs.push("0".to_string());
                    } else {
                        let mut token = m.as_str().to_owned();
                        for c in REGEX_STUFF {
                            token = token.replace(c, format!("\\{}", c).as_str());

                        }
                        temp_combination = tokens[1].to_owned();
                        temp_inputs.push(token);
                    }
                }
                if temp_inputs.join("").len() < combinations.len() {
                    temp_inputs.push("0".to_string());
                }
            }
            mapped_inputs.push(temp_inputs.clone());
            Some(temp_inputs)
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
                    if self.get_images {
                        self.get_images = false;
                        self.retained_images.clear();
                        for (_k, v) in self.game_json.as_ref().unwrap()["attacks"]
                            .as_object()
                            .unwrap()
                            .iter()
                        {
                            if v != "skip" {
                                let mut buffer = vec![];
                                let v = v.to_string().remove_quotes();
                                let path = format!("images\\{}", v);
                                File::open(&path).unwrap().read_to_end(&mut buffer).unwrap();
                                let retained = RetainedImage::from_image_bytes(v, &buffer).unwrap();
                                self.retained_images.push(Some(retained));

                            }
                        }
                        for (_k, v) in self.default_json["movement"].as_object().unwrap().iter() {
                            if v != "skip" {
                                let mut buffer = vec![];
                                let v = v.to_string().remove_quotes();
                                let path = format!("images\\{}", v);
                                File::open(&path).unwrap().read_to_end(&mut buffer).unwrap();
                                let retained = RetainedImage::from_image_bytes(v, &buffer).unwrap();
                                self.retained_images.push(Some(retained));

                            }
                        }
                    }

                    // ui.label("Contents");
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            let temp_selection = self.game_selected.clone();
                            egui::ComboBox::from_id_source("character_select")
                                .selected_text(
                                    self.game_selected
                                        .to_owned()
                                        .unwrap_or("Select a game.".to_string()),
                                )
                                .show_ui(ui, |ui| {
                                    match self.game_list.to_owned().unwrap() {
                                        Value::Object(obj) => {
                                            for (k, _v) in obj.iter() {
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
                                        self.get_images = true;
                                        self.changed_inputs = true;
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
                                if let Some(_selected) = self
                                    .game_list
                                    .to_owned()
                                    .unwrap()
                                    .get(self.game_selected.to_owned().unwrap())
                                {
                                    ui.end_row();
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::from_id_source("character_box")
                                            .selected_text(if self.character_selected.is_some() {
                                                self.character_selected
                                                    .as_ref()
                                                    .unwrap()
                                                    .name
                                                    .clone()
                                            } else {
                                                "Select a character".to_string()
                                            })
                                            .show_ui(ui, |ui| {
                                                if self.character_list.is_none() {
                                                    get_character_list(self);
                                                } else {
                                                    for c in self.character_list.as_ref().unwrap() {
                                                        let c_name =
                                                            c.name.to_owned().remove_quotes();
                                                        if ui
                                                            .selectable_value(
                                                                &mut self.character_selected,
                                                                Some(c.to_owned()),
                                                                c_name,
                                                            )
                                                            .changed()
                                                        {
                                                            // println!("changed char");
                                                            self.changed_inputs = true;
                                                            self.mapped_inputs.clear();
                                                        };
                                                    }
                                                }
                                                if let Some(selected) =
                                                    self.character_selected.as_ref()
                                                {
                                                    if Some(selected.name.clone())
                                                        != self.previous_choice
                                                    {
                                                        self.combo_selector = 0.0;
                                                        if !self
                                                            .character_selected
                                                            .as_ref()
                                                            .unwrap()
                                                            .combos
                                                            .is_empty()
                                                        {
                                                            self.inputs = self
                                                                .character_selected
                                                                .as_ref()
                                                                .unwrap()
                                                                .combos
                                                                [self.combo_selector as usize]
                                                                .to_owned()
                                                                .unwrap()
                                                                .inputs;
                                                        }
                                                    }
                                                }
                                            });
                                    });
                                }
                            }
                        });

                        //ui.add_space(70.00);
                        egui::CollapsingHeader::new("ADD").show(ui, |ui| {
                            if ui.button("ADD COMBO").clicked() {
                                self.show_window = !self.show_window;
                            }
                            if self.show_window {
                                egui::Window::new("ADD NEW COMBO")
                                    .collapsible(false)
                                    .min_width(WIDTH + 20.0)
                                    .auto_sized()
                                    .fixed_pos(Pos2::new(0.0, 0.0))
                                    .show(ctx, |ui| {
                                        ui.label(
                                            self.game_selected
                                                .as_ref()
                                                .unwrap_or(&"what".to_string()),
                                        );
                                        ui.horizontal(|ui| {
                                            egui::ComboBox::from_label("")
                                                .selected_text(
                                                    if self.character_selected.is_some() {
                                                        self.character_selected
                                                            .as_ref()
                                                            .unwrap()
                                                            .name
                                                            .clone()
                                                    } else {
                                                        "Select a character".to_string()
                                                    },
                                                )
                                                .show_ui(ui, |ui| {
                                                    if self.character_list.is_none() {
                                                        get_character_list(self);
                                                    } else {
                                                        for c in
                                                            self.character_list.as_ref().unwrap()
                                                        {
                                                            let c_name =
                                                                c.name.to_owned().remove_quotes();
                                                            ui.selectable_value(
                                                                &mut self.character_selected,
                                                                Some(c.to_owned()),
                                                                c_name,
                                                            );
                                                        }
                                                    }
                                                })
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            let game_name = ui.label("Name: ");
                                            ui.text_edit_singleline(&mut self.description)
                                                .labelled_by(game_name.id);
                                        });
                                        ui.horizontal_wrapped(|ui| {
                                            let inputs = ui.label("Inputs: ");
                                            ui.text_edit_multiline(&mut self.new_inputs)
                                                .labelled_by(inputs.id);
                                        });

                                        ui.add_space(10.0);
                                        ui.horizontal_wrapped(|ui| {
                                            if ui.button("ADD TEST").clicked() {
                                                let temp_combo = Combo::new(
                                                    self.description.to_owned(),
                                                    self.new_inputs.to_ascii_uppercase(),
                                                    ComboState::Testing,
                                                );
                                                add_combo(
                                                    self.character_selected.to_owned().unwrap(),
                                                    self,
                                                    temp_combo,
                                                );
                                                self.new_inputs = "".to_owned();
                                                self.description = "".to_owned();
                                                self.show_window = false;
                                            };
                                            if ui.button("CANCEL").clicked() {
                                                self.new_inputs = "".to_owned();
                                                self.description = "".to_owned();
                                                self.show_window = false;
                                            };
                                        });
                                    });
                            }
                        });
                    });
                });
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.character_selected.is_some()
                    && !self.character_selected.as_ref().unwrap().combos.is_empty()
                {
                    let slider_size: f32 =
                        self.character_selected.as_ref().unwrap().combos.len() as f32 - 1.0;
                    if slider_size < self.combo_selector {
                        self.combo_selector = slider_size
                    };
                    let selected_combos = self.character_selected.as_ref().unwrap().combos
                        [self.combo_selector as usize]
                        .to_owned()
                        .unwrap();
                    if ui
                        .add(
                            egui::Slider::new(&mut self.combo_selector, 0.0..=slider_size)
                                .step_by(1.0)
                                .fixed_decimals(0)
                                .text(selected_combos.name),
                        )
                        .changed()
                    {
                        self.changed_inputs = true;
                        self.mapped_inputs.clear();
                        self.inputs = self.character_selected.as_ref().unwrap().combos
                            [self.combo_selector as usize]
                            .clone()
                            .unwrap()
                            .inputs;
                    };
                }
                egui::CollapsingHeader::new("INPUTS").show(ui, |ui| {
                    let name_label = ui.label("Inputs: ");
                    if ui
                        .text_edit_multiline(&mut self.inputs)
                        .labelled_by(name_label.id)
                        .changed()
                    {
                        self.inputs = self.inputs.to_ascii_uppercase();
                        self.changed_inputs = true;
                        self.mapped_inputs.clear();
                    };
                });
                if ui.button("Toggle").clicked() {
                    self.show_images = !self.show_images;
                }
                if self.show_images && !self.inputs.is_empty() && self.changed_inputs {
                    self.changed_inputs = false;
                    self.inputs = str::replace(&self.inputs, ',', " ");
                    let mut translate: Vec<String> = Vec::new();
                    for attack in self.game_json.as_ref().unwrap()["attacks"]
                        .as_object()
                        .unwrap()
                        .iter()
                    {
                        translate.push(attack.0.to_owned());
                    }
                    for movement in self.default_json["movement"].as_object().unwrap().iter() {
                        translate.push(movement.0.to_owned());
                    }
                    // println!("{:?}", translate);
                    //println!("{}", self.json["attacks"]);
                    let inputs = self.inputs.split('\n');
                    for combinations in inputs {
                        let mut combinations: String = combinations.to_owned();
                        self.mapped_inputs
                            .push(compare_combinations(&mut combinations, &mut translate));
                    }
                } else if !self.show_images {
                    self.changed_inputs = true;
                    self.mapped_inputs.clear();
                }
                // println!("{:?}", self.mapped_inputs);
                for translation in self.mapped_inputs.iter_mut().flatten() {
                    ui.horizontal_wrapped(|ui| {
                        for input in translation.iter_mut() {
                            // println!("651: {}", input);
                            if let Some(m) = self.game_json.as_ref().unwrap()["attacks"]
                                .as_object()
                                .unwrap()
                                .get(input)
                                .or(self.default_json["movement"]
                                    .as_object()
                                    .unwrap()
                                    .get(input))
                            {
                                // find the index that has m in self.retained_images
                                let idx = self.retained_images.iter().position(|x| {
                                    // println!("{} - {}", x.as_ref().unwrap().debug_name(), m.to_string().remove_quotes());
                                    // println!("{} - {}", x.as_ref().unwrap().debug_name(), m.to_string().remove_quotes());
                                    x.as_ref().unwrap().debug_name()
                                        == m.to_string().remove_quotes()
                                });
                                if let Some(idx) = idx {
                                    self.retained_images[idx].as_ref().unwrap().show(ui);
                                }
                            } else {
                                // println!("{}", self.json_default["movement"]["_"]);
                                if let Some(m) = self.default_json["movement"].get("_") {
                                    let idx = self.retained_images.iter().position(|x| {
                                        // println!("{} - {}", x.as_ref().unwrap().debug_name(), "default\\\\err.png".to_string().remove_quotes());
                                        x.as_ref().unwrap().debug_name().trim()
                                            == m.to_string().remove_quotes()
                                    });
                                    if let Some(idx) = idx {
                                        self.retained_images[idx].as_ref().unwrap().show(ui);
                                    }
                                }
                            }
                        }
                        // println!("__");
                        // add egui separator
                    });
                    ui.vertical(|ui| {
                        ui.separator();
                    });
                }

                if let Some(previous) = self.character_selected.as_ref() {
                    self.previous_choice = Some(previous.name.to_owned());
                } else {
                    self.previous_choice = Some("Select a character".to_string());
                }

                //ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });
        });
    }
}
