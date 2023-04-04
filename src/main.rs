#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::{ColorImage, Label};
use egui_extras::RetainedImage;
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
        initial_window_size: Some(egui::vec2(340.0, 440.0)),
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

struct MyApp {
    image: Option<RetainedImage>,
    inputs: String,
    game_name: String,
    age: u32,
    texture: Option<egui::TextureHandle>,
    render_input: bool,
    new_line: bool,
    read_game_list: bool,
    game_list: Option<Value>,
    game_selected: Option<String>,
    read_character_list: bool,
    character_list: Option<Value>,
    character_selected: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            image: None,
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
            age: 42,
            texture: None,
            new_line: false,
            render_input: false,
            read_game_list: true,
            game_list: None,
            game_selected: None,
            read_character_list: true,
            character_list: None,
            character_selected: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::CollapsingHeader::new("GAME OPTIONS")
                .default_open(true)
                .show(ui, |ui| {
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
                                    if self.read_game_list {
                                        self.read_game_list = false;
                                        let games_list_str = fs::read_to_string(Path::new(
                                            "src\\games\\game_list.json",
                                        ))
                                        .expect("Unable to read game_list_name.json");
                                        self.game_list = serde_json::from_str(&games_list_str)
                                            .expect("bad json.");
                                    } else {
                                        match self.game_list.to_owned().unwrap() {
                                            Value::Object(obj) => {
                                                for (k, v) in obj.iter() {
                                                    ui.selectable_value(
                                                        &mut self.game_selected,
                                                        Some(k.clone()),
                                                        k,
                                                    );
                                                }
                                            }
                                            _ => {
                                                println!("json is not an object.");
                                            }
                                        }
                                    }
                                    self.read_character_list = self.game_selected != temp_selection;
                                    println!("{}", self.read_character_list);
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
                                                self.character_selected
                                                    .to_owned()
                                                    .unwrap_or("Select a character.".to_string()),
                                            )
                                            .show_ui(ui, |ui| {
                                                if self.read_character_list {
                                                    self.read_character_list = false;
                                                    if let Some(selected) =
                                                        self.game_list.to_owned().unwrap().get(
                                                            self.game_selected.to_owned().unwrap(),
                                                        )
                                                    {
                                                        let selected =
                                                            selected.to_string().replace('\"', "");
                                                        let character_list_str =
                                                            fs::read_to_string(Path::new(
                                                                &format!(
                                                                    "src\\games\\input_{}.json",
                                                                    selected
                                                                ),
                                                            ))
                                                            .expect("unable to read input_().json");
                                                        self.character_list = serde_json::from_str(
                                                            &character_list_str,
                                                        )
                                                        .expect("bad input_().json");
                                                        self.character_list = self
                                                            .character_list
                                                            .to_owned()
                                                            .unwrap()
                                                            .get("characters")
                                                            .cloned();
                                                    }
                                                } //else {
                                                    match self.character_list.to_owned().unwrap() {
                                                        Value::Object(obj) => {
                                                            for (k, v) in obj.iter() {
                                                                let v =
                                                                    v.to_string().replace('\"', "");
                                                                ui.selectable_value(
                                                                    &mut self.character_selected,
                                                                    Some(v.clone()),
                                                                    v,
                                                                );
                                                            }
                                                        }
                                                        _ => {
                                                            println!("json is not an object.");
                                                        }
                                                    }
                                                //}
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
                                            let selected = selected.to_string().replace('\"', "");
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
