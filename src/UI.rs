use std::{fs::{self, File}, path::Path};

use eframe::{run_native, epi::App, egui::{self}};
use ::egui::color::srgba;
use crate::db_config::{self, Users};
use mongodb::{Client, Collection  , Database};
use mongodb::bson::{self,oid::ObjectId};
use tokio::{io, time::Instant, runtime::Runtime};
use tokio;
use std::io::BufReader;
use rodio::{OutputStream, Sink, source::Buffered};
use std::{process::Command};

use db_config::{connect_to_mongodb, VoiceNote};
use record_audio::audio_clip::AudioClip as ac;

// Enum to represent different pages/screens of the application
enum Page {
    Signup,
    Login,
    Home,
    MyProfile,
    MyPost,
    FollowerPost,
}

pub struct Gui {
    // Current page of the application
    current_page: Page,

    // Error message to display
    error_message: Option<String>,
    user_collection: Option<Collection<db_config::Users>>,
    voice_note_collection: Option<Collection<db_config::VoiceNote>>,
    database: Option<Database>,
    client: Option<Client>,

    // Data for login and signup page
    username: String,
    password: String,
    email: String,
    confirm_password: String,
    userid: ObjectId,

    // Extra data
    theme: Theme,
}

// Define an enum to represent the current theme/mode
enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark // Set the default theme/mode to dark
    }
}


impl Gui {
    fn toggle_theme(&mut self, ctx: &egui::CtxRef) {
        match self.theme {
            Theme::Dark => {
                self.theme = Theme::Light;
                ctx.set_visuals(egui::Visuals::light());
            }
            Theme::Light => {
                self.theme = Theme::Dark;
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }

    pub fn new() -> Self {
        Self {
            current_page: Page::Login,
            error_message: None,
            username: String::new(),
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            theme: Theme::default(),
            user_collection: None,
            voice_note_collection: None,
            database: None,
            client: None,
            userid: ObjectId::new(),
        }
    }

    // Function to show the signup page UI
    fn signup_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        ui.heading("Sign up");
        ui.add_space(10.0);
        // Group the contents of the signup page
        ui.group(|ui| {
            // Calculate available width for each column
            let column_width = ui.available_width();
            ui.horizontal(|ui| {
                ui.label("Username: ");
                let current_width = ui.available_width();
                ui.add_space(110.0-(column_width-current_width)); // Add spacing between the heading and the buttons
                ui.text_edit_singleline(&mut self.username);
            });
            ui.horizontal(|ui| {
                ui.label("Email: ");
                let current_width = ui.available_width();
                ui.add_space(110.0-(column_width-current_width)); // Add spacing between the label and the text edit
                ui.text_edit_singleline(&mut self.email);
            });
            ui.horizontal(|ui| {
                ui.label("Password: ");
                let current_width = ui.available_width();
                ui.add_space(110.0-(column_width-current_width)); // Add spacing between the heading and the buttons
                ui.text_edit_singleline(&mut self.password);
            });
            ui.horizontal(|ui| {
                ui.label("Confirm Password: ");
                let current_width = ui.available_width();
                ui.add_space(110.0-(column_width-current_width)); // Add spacing between the heading and the buttons
                ui.text_edit_singleline(&mut self.confirm_password);
            });
            if ui.button("Sign up").clicked() {
                
                // Handle signup button click
                if self.password != self.confirm_password {
                    // Show an error message if the passwords don't match
                    self.error_message = Some("Passwords don't match".to_string()); // Store error message in a variable
                } else if self.username.is_empty() || self.password.is_empty() || self.email.is_empty() {
                    // Show an error message if either the username or password is empty
                    self.error_message = Some("All fields are required".to_string()); // Store error message in a variable
                } else 
                {
                    let rt= Runtime::new().unwrap();
                    let username= self.username.clone();
                    let email= self.email.clone();
                    let pass= self.password.clone();
                    let (usercol, voicecol, database, client) = rt.block_on( async move
                        {
                            let response = tokio::spawn
                            ( async move
                                {
                                    let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                    (user_collection, voice_note_collection, db, client)
                                }
                            ).await.unwrap();
                            response
                        });
                    self.user_collection = Some(usercol.clone());
                    self.voice_note_collection = Some(voicecol.clone());
                    self.database = Some(database.clone());
                    self.client = Some(client.clone());

                    let another_uc = self.user_collection.clone().unwrap();

                    let response = rt.block_on( async move
                        {
                            let response = tokio::spawn
                            ( async move
                                {
                                    let response = db_config::create_user(another_uc, email, pass, username).await;

                                    response
                                }
                            ).await.unwrap();
                            response
                        });
                    self.userid = response;
                    self.current_page = Page::Login;

                }
            }
            if let Some(error_message) = &self.error_message {
                // Display error message if it exists
                ui.add(egui::Label::new(error_message).text_color(egui::Color32::RED));
            }
            ui.horizontal(|ui| {
                ui.label("Already have an account?");
                if ui.button("Log in").clicked() {
                    // Show the login page if the user clicks the login button
                    self.username.clear();
                    self.password.clear();
                    self.confirm_password.clear();
                    self.error_message = None;
                    self.current_page = Page::Login;
                }
            });
        });
    }



    // Function to show the login page UI
    fn login_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        ui.heading("Login");
        ui.add_space(10.0);
        // Group the contents of the login page
        ui.group(|ui| {
            // Calculate available width for each column
            let column_width = ui.available_width();
            ui.horizontal(|ui| {
                ui.label("Username: ");
                let current_width = ui.available_width();
                ui.add_space(70.0-(column_width-current_width)); // Add spacing between the label and the text edit
                ui.text_edit_singleline(&mut self.username);
            });
            ui.horizontal(|ui| {
                ui.label("Password: ");
                let current_width = ui.available_width();
                ui.add_space(70.0-(column_width-current_width)); // Add spacing between the label and the text edit
                ui.text_edit_singleline(&mut self.password);
            });
            if ui.button("Log in").clicked() 
            {
                // Handle login button click
                // -----------------------------------login implementation
                // if self.username == "admin" && self.password == "password" {
                //     self.current_page = Page::Home;
                // } else {
                //     // Show an error message if the login failed
                //     self.error_message = Some("login failed. Incorrect user name and password".to_string()); // Store error message in a variable
                // }
                let rt= Runtime::new().unwrap();
                let username= self.username.clone();
                let pass= self.password.clone();
                let (usercol, voicecol, database, client) = rt.block_on( async move
                    {
                        let response = tokio::spawn
                        ( async move
                            {
                                let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                (user_collection, voice_note_collection, db, client)
                            }
                        ).await.unwrap();
                        response
                    });
                self.user_collection = Some(usercol.clone());
                self.voice_note_collection = Some(voicecol.clone());
                self.database = Some(database.clone());
                self.client = Some(client.clone());

                let another_uc = usercol.clone();
                let vc = voicecol.clone();

                let response = rt.block_on( async move
                    {
                        let response = tokio::spawn
                        ( async move
                            {
                                let response = db_config::get_user_by_username(another_uc.clone(), username, pass).await;
                                let user_iddd= response.unwrap()._id.clone();
                                let response2 = db_config::get_all_voice_ids_from_following(another_uc ,vc , user_iddd).await;
        
                                (user_iddd,response2)
                            }
                        ).await.unwrap();
                        response
                    });
                self.userid = response.0.clone();
                println!("login successful : {:?}",response.0.clone() );

                self.current_page = Page::Home;
                
            }
                if let Some(error_message) = &self.error_message {
                    // Display error message if it exists
                    ui.add(egui::Label::new(error_message).text_color(egui::Color32::RED));
                }
                ui.horizontal(|ui| {
                    ui.label("Don't have an account?");
                    if ui.button("Sign up").clicked() {
                        // Show the signup page if the user clicks the sign up button
                        self.error_message = None;
                        self.username.clear();
                        self.password.clear();
                        self.current_page = Page::Signup;
                    }
                });
            });
    }

    



// Function to show the home page UI
fn home_page(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
    

    let folder_name = format!("{}", self.userid);
    fs::create_dir_all(&folder_name).unwrap(); 
    ui.heading("Voicer Home Page");
    ui.add_space(10.0);
    ui.label(format!("Welcome, {}!", self.username)); // Display welcome message with user's name

    // Count the number of voicenotes
    let voicenote_count = count_voicenotes();
    ui.label(format!("You have {} voicenotes.", voicenote_count));

    ui.add_space(10.0);

    // Display voicenote posts
    for i in 0..voicenote_count {
        ui.collapsing(format!("Voicenote {}", i + 1), |ui| {
            // Add content for each voicenote post here
            ui.label("Voicenote content");

            // Play single voicenote button
            ui.horizontal(|ui| {
                if ui.button("▶️ Play").clicked() {
                    // Play the selected voicenote
                    let directory = "."; // Main directory path
                    let files = fs::read_dir(directory).unwrap();
                    let mut filenames: Vec<String> = Vec::new(); // Vector to store filenames

                    for file in files {
                        if let Ok(file) = file {
                            if let Some(extension) = file.path().extension() {
                                if extension == "wav" {
                                    let filename = file.path().to_str().unwrap().to_owned();
                                    filenames.push(filename);
                                }
                            }
                        }
                    }

                    // Play the audio file
                    if let Some(filename) = filenames.get(i) {
                         play_audio(filename);
                        // if ui.button("⏸️ Pause").clicked() {
                        //     // Pause the currently playing audio
                        //     pause_audio(&x);
                        // }

                        // if ui.button("⏹️ Stop").clicked() {
                        //     // Stop the audio playback
                        //     stop_audio(&x);
                        // }
                    }
                }
            });
        });
    }

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        if ui.button("▶️ Play All").clicked() {
            // Play all voicenote files
            let directory = "."; // Main directory path
            let files = fs::read_dir(directory).unwrap();
            let mut filenames: Vec<String> = Vec::new(); // Vector to store filenames

            for file in files {
                if let Ok(file) = file {
                    if let Some(extension) = file.path().extension() {
                        if extension == "wav" {
                            let filename = file.path().to_str().unwrap().to_owned();
                            filenames.push(filename);
                        }
                    }
                }
            }

            // Play the audio files
            for filename in filenames {
            play_audio(&filename);

                // ui.horizontal(|ui| {
                //     if ui.button("⏸️ Pause").clicked() {
                //         // Pause the currently playing audio
                //         pause_audio(&x);
                //     }

                //     if ui.button("⏹️ Stop").clicked() {
                //         // Stop the audio playback
                //         stop_audio(&x);
                //     }
                // });
            }
        }

        if ui.button("📣 Tweet").clicked() {
            // Tweet the voicenote
            self.current_page=Page::MyProfile;                        
        }
        if ui.button("Theme").clicked() {
            // Change mode to light mode
            self.toggle_theme(ctx);
        }

        if ui.button("Logout").clicked() {
            // Redirect to Login page and clear user data
            self.username.clear();
            self.password.clear();
            self.confirm_password.clear();
            self.error_message = None;
            self.current_page = Page::Login;
        }
    });
}




    // Function to show the My Files page UI
    fn my_files_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        // Calculate available width for each column
        let column_width = ui.available_width();
        ui.horizontal(|ui| {            
            ui.heading("My Files"); // Display heading "My Files"

            let current_width = ui.available_width();
            ui.add_space(600.0-(column_width-current_width)); // Add spacing between the heading and the buttons
            if ui.button("Upload").clicked() {
                // Handle "Upload" button click
            }
            ui.spacing();
            if ui.button("home").clicked() {
                self.current_page = Page::Home;
            }
        });
        ui.add_space(10.0);

        // Example data for the table
        let file_names = vec![
            "File 1", "File 2", "File 3", "File 4", "File 5",
            "File 6", "File 7", "File 8", "File 9", "File 10",
            "File 11", "File 12", "File 13", "File 14", "File 15",
            "File 16", "File 17", "File 18", "File 19", "File 20",
        ];

        let upload_dates = vec![
            "2023-04-01", "2023-03-28", "2023-03-25", "2023-03-22", "2023-03-19",
            "2023-03-16", "2023-03-13", "2023-03-10", "2023-03-07", "2023-03-04",
            "2023-03-01", "2023-02-26", "2023-02-23", "2023-02-20", "2023-02-17",
            "2023-02-14", "2023-02-11", "2023-02-08", "2023-02-05", "2023-02-02",
        ];

        let file_sizes = vec![
            "10 MB", "25 MB", "5 MB", "50 MB", "100 MB",
            "20 MB", "15 MB", "30 MB", "40 MB", "75 MB",
            "12 MB", "18 MB", "8 MB", "60 MB", "90 MB",
            "22 MB", "35 MB", "45 MB", "80 MB", "70 MB",
        ];
    
        // let file_members = vec![
        //     "m.ahsan@gmail.com", "m.ahsan@gmail.com", "j.doe@gmail.com", "j.doe@gmail.com", "m.ahsan@gmail.com", "j.doe@gmail.com", "j.doe@gmail.com",
        //     "m.ahsan@gmail.com", "j.doe@gmail.com", "j.doe@gmail.com", "m.ahsan@gmail.com", "j.doe@gmail.com", "j.doe@gmail.com", "m.ahsan@gmail.com",
        //     "m.ahsan@gmail.com" , "j.doe@gmail.com", "j.doe@gmail.com", "m.ahsan@gmail.com", "j.doe@gmail.com", "j.doe@gmail.com", 
        // ];

        // Begin table
        egui::ScrollArea::auto_sized().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name"); // Column 1: Name

                let current_width = ui.available_width();
                ui.add_space(400.0-(column_width-current_width)); // Add spacing between columns

                ui.label("Upload Date"); // Column 2: Upload Date

                let current_width = ui.available_width();
                ui.add_space(500.0-(column_width-current_width)); // Add spacing between columns

                ui.label("Size"); // Column 3: Size

                // let current_width = ui.available_width();
                // ui.add_space(500.0-(column_width-current_width)); // Add spacing between columns

                // ui.label("Members"); // Column 4: Members

                let current_width = ui.available_width();
                ui.add_space(600.0-(column_width-current_width)); // Add spacing between columns

                ui.label("Options"); // Column 5: Options
            });

            // Display table rows
            for i in 0..file_names.len() {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(file_names[i]); // Display file name in Column 1

                    let current_width = ui.available_width();
                    ui.add_space(400.0-(column_width-current_width)); // Add spacing between columns

                    ui.label(upload_dates[i]); // Display last modified date in Column 2

                    let current_width = ui.available_width();
                    ui.add_space(500.0-(column_width-current_width)); // Add spacing between columns

                    // Display file size in Column 3
                    ui.label(file_sizes[i]); // Display "file_sizes[i]"

                    // let current_width = ui.available_width();
                    // ui.add_space(500.0-(column_width-current_width)); // Add spacing between columns

                    // // Display members in Column 4
                    // ui.label(file_members[i]); // Display "shared_with[i]"

                    let current_width = ui.available_width();
                    ui.add_space(600.0-(column_width-current_width)); // Add spacing between columns

                    // Display buttons in Column 5 with updated labels
                    if ui.button("Download").clicked() {
                        // Handle "Download" button click
                    }
                    ui.spacing(); // Add spacing between buttons
                    if ui.button("Share").clicked() {
                        // Handle "Share" button click
                    }
                    ui.spacing(); // Add spacing between buttons
                    if ui.button("Delete").clicked() {
                        // Handle "Delete" button click
                    }
                });
            }

            // End table
            ui.separator();
        });
    }

    // Function to show the Shared Files page UI
    fn shared_files_page_1(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        // Calculate available width for each column
        let column_width = ui.available_width();
        ui.horizontal(|ui| {
            ui.heading("Shared Files"); // Display heading "Shared Files"
            let current_width = ui.available_width();
            ui.add_space(600.0-(column_width-current_width)); // Add spacing between header and button
            if ui.button("home").clicked() {
                self.current_page = Page::Home;
            }
        });
        ui.add_space(10.0);        
    
        // Example data for the table
        let names = vec!["Person 1", "Person 2", "Person 3"];
        let items= vec!["1", "3", "2"];
        let last_modified = vec!["2023-04-01", "2023-03-28", "2023-03-25"];
    
        // Begin table
        egui::ScrollArea::auto_sized().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name"); // Column 1: Name

                let current_width = ui.available_width();
                ui.add_space(300.0-(column_width-current_width)); // Add spacing between columns
                ui.label("Items"); // Column 2: Items

                let current_width = ui.available_width();
                ui.add_space(600.0-(column_width-current_width)); // Add spacing between columns
                ui.label("Last modified"); // Column 3: Last modified
            });

            // Display table rows
            for i in 0..names.len() {
                ui.separator();
                ui.horizontal(|ui| {
                    // ui.add_space(column_width); // Add empty space for first column
                    if ui.button(format!("{}",names[i])).clicked() {
                        //self.current_page = Page::SharedFiles2;
                    } // Display file name in Column 1

                    let current_width = ui.available_width();
                    ui.add_space(300.0-(column_width-current_width)); // Add spacing between columns
                    ui.label(items[i]); // Display shared by in Column 2

                    let current_width = ui.available_width();
                    ui.add_space(600.0-(column_width-current_width)); // Add spacing between columns
                    ui.label(last_modified[i]); // Display last modified date in Column 3
                });
            }
        });
    }
        // Function to show the Shared Files page UI
        fn shared_files_page_2(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
            // Calculate available width for each column
            let column_width = ui.available_width();
            ui.horizontal(|ui| {
                ui.heading("Shared Files with person x"); // Display heading "Shared Files"
                let current_width = ui.available_width();
                ui.add_space(600.0-(column_width-current_width)); // Add spacing between header and button
                if ui.button("back").clicked() {
                    //self.current_page = Page::SharedFiles;
                }
                if ui.button("home").clicked() {
                    self.current_page = Page::Home;
                }
            });
            ui.add_space(10.0);        
        
            // Example data for the table
            let file_names = vec!["File 1", "File 2", "File 3"];
            // let shared_by = vec!["m.ahsan@gmail", "maaz.shamim@hotmail", "maaz.batla@outlook"];
        
            // Begin table
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name"); // Column 1: Name
    
                    // let current_width = ui.available_width();
                    // ui.add_space(300.0-(column_width-current_width)); // Add spacing between columns
                    // ui.label("Shared by"); // Column 2: Shared by
    
                    let current_width = ui.available_width();
                    ui.add_space(600.0-(column_width-current_width)); // Add spacing between columns
                    ui.label("Options"); // Column 3: Options
                });
    
                // Display table rows
                for i in 0..file_names.len() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        // ui.add_space(column_width); // Add empty space for first column
                        ui.label(file_names[i]); // Display file name in Column 1
    
                        // let current_width = ui.available_width();
                        // ui.add_space(300.0-(column_width-current_width)); // Add spacing between columns
                        // ui.label(shared_by[i]); // Display shared by in Column 2
    
                        let current_width = ui.available_width();
                        ui.add_space(600.0-(column_width-current_width)); // Add spacing between columns
    
                        if ui.button("Download").clicked() { // Display buttons in Column 3
                            // Handle "Download" button click
                        }
                        ui.spacing(); // Add spacing between buttons
                        if ui.button("Share").clicked() {
                            // Handle "Share" button click
                        }
                    });
                }
            });
        }
    
}

impl App for Gui {
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut eframe::epi::Frame<'_>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Signup => {
                    self.signup_page(ctx, ui);
                }
                Page::Login => {
                    self.login_page(ctx, ui);
                }
                Page::Home => {
                    self.home_page(ctx, ui);
                }
                Page::MyProfile => {
                    self.my_files_page(ctx, ui);
                }
                Page::MyPost => {
                    self.shared_files_page_1(ctx, ui);
                },
                Page::FollowerPost => {
                    self.shared_files_page_2(ctx, ui);
                }
            }
        });
    }

    fn name(&self) -> &str {
        "Voicer"
    }
}

// Function to count the number of .wav files in the main directory
fn count_voicenotes() -> usize {
    let directory = "."; // Main directory path
    let files = fs::read_dir(directory).unwrap();
    let mut count = 0;

    for file in files {
        if let Ok(file) = file {
            if let Some(extension) = file.path().extension() {
                if extension == "wav" {
                    count += 1;
                }
            }
        }
    }

    count
}

use rodio::{source::Source};

// Function to play the audio and return the Sink object
fn ad_play_audio(filename: &str) -> Sink {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open(filename).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap().buffered();
    
    sink.append(source);
    sink.play();

    sink
}

fn play_audio(filename: &str) {
    // Load the voice note file
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open(filename).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    // Play the voice note
    sink.append(source);
    sink.sleep_until_end();
}

// Function to pause the audio playback
fn pause_audio(sink: &Sink) {
    sink.pause();
}

// Function to stop the audio playback
fn stop_audio(sink: &Sink) {
    sink.stop();
}

async fn save_audio() {

}