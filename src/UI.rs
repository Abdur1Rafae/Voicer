use std::{fs::{self, File}, path::Path, vec};
use chrono::{DateTime, Utc, TimeZone};
use eframe::{run_native, epi::App, egui::{self}};
use ::egui::color::srgba;
use crate::db_config::{self, Users, publicUser, get_user_by_username, find_users_by_names};
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
    MyTweet,
    Follow,
    FollowerPost,
}

pub struct Gui {
    // Current page of the application
    current_page: Page,

    // Error message to display
    error_message: Option<String>,
    user_collection: Option<Collection<db_config::Users>>,
    userslist: ObjectId,
    voice_note_collection: Option<Collection<db_config::VoiceNote>>,
    database: Option<Database>,
    client: Option<Client>,
    voicenote_vec: Option<Vec<db_config::VoiceNote>>,

    // Data for login and signup page
    username: String,
    followuser: String,
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
            followuser:String::new(),
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            theme: Theme::default(),
            voicenote_vec: None,
            user_collection: None,
            voice_note_collection: None,
            userslist : ObjectId::new(),
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
                self.voicenote_vec = Some(response.1.clone());
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
            self.current_page=Page::MyTweet;                        
        }
        if ui.button("➹ Follow").clicked() {
            // Follow a user
            self.current_page=Page::Follow;                        
        }
        if ui.button("Theme").clicked() {
            // Change mode to light mode
            self.toggle_theme(ctx);
        }

        if ui.button("Logout").clicked() {
            // Redirect to Login page and clear user data
            if let Err(err) = delete_wav_files() {
                eprintln!("Error deleting .wav files: {}", err);
            } else {
                println!("Successfully deleted .wav files");
            }        
            self.username.clear();
            self.password.clear();
            self.confirm_password.clear();
            self.error_message = None;
            self.current_page = Page::Login;
        }
    });

    ui.add_space(10.0);

    let mut vec_vc = self.voicenote_vec.clone();
    egui::ScrollArea::auto_sized().show(ui, |ui| {

        // Display voicenote posts
        for i in 0..voicenote_count {
            let voice_obj = vec_vc.clone().unwrap()[i].clone();
            ui.group(|ui| {
                ui.label(format!("Voicenote {} by {}",i+1, voice_obj.name));
                // Add content for each voicenote post here

                // Play single voicenote button
                ui.horizontal(|ui| {
                    if ui.button("▶️ Play").clicked() {
                        let filename = voice_obj._id.to_hex()+".wav";
                        let x = play_audio(&filename);
                    }
                });

                let time = Utc.timestamp(voice_obj.timestamp.timestamp(), 0);
                let formatted_time = time.format("%Y-%m-%d %H:%M:%S").to_string();

                ui.label(format!("Posted on: {}", formatted_time));

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Shut Up").clicked() {
                            
                        }
                        if ui.button("Speak Up").clicked() {
    
                        }
                        if ui.button("Conversation").clicked() {
                        
                        }
                    });
                });

                
            });
        }
    });

    ui.add_space(10.0);
}




fn tweet_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
    let column_width = ui.available_width();
    let folder_name = format!("{}", self.userid);
    fs::create_dir_all(&folder_name).unwrap();
    let mut file_name = ObjectId::new();
    let directory = format!("{}/{}.wav", folder_name, file_name.to_hex());
    let mut is_saved = false; // Flag to indicate if voicenote is successfully saved

    ui.horizontal(|ui| {
        ui.heading("Your voicenote can now be recorded...");

        let current_width = ui.available_width();
        ui.add_space(600.0 - (column_width - current_width));
        
        // Record button
        if ui.button("Record").clicked() {
            match ac::record(None) {
                Ok(clip) => {
                                // Stop button
                    if ui.button("Stop").clicked() {
                        println!("Stopped recording.");match clip.export(format!("{}", directory).as_str()) {
                            Ok(_) => {
                                println!("Successfully saved!");
                                is_saved = true;
                            }
                            Err(err) => println!("Error {}", err),
                        }}
                        // Perform any necessary cleanup or stopping logic here
                        // For example, you could terminate the recording process or close any open file handles.}
                }
                Err(err) => println!("Error {}", err),
            }
            self.current_page=Page::Home;
        }

        

        // Display success message if voicenote is saved
        if is_saved {
            ui.label("Voicenote saved successfully!");
        }

        // Back button
        ui.spacing();
        if ui.button("Back").clicked() {
            self.current_page = Page::Home;
        }
    });

    ui.add_space(10.0);

    }
    // Function to show the Shared Files page UI
fn FollowPage(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        // ui.horizontal(|ui| {
        //     ui.heading("Follow a Voicer User"); // Display heading "Shared Files"
        //     let current_width = ui.available_width();
        //     if ui.button("home").clicked() {
        //         self.current_page = Page::Home;
        //     }
        // let mut fuser = String::new();
        // let mut fusernum = String::new();

        //     ui.label("Username: ");
        //     let current_width = ui.available_width();
        //     ui.text_edit_singleline(&mut fuser);

        // if ui.button("Follow").clicked() {
        //         // getuserlist(fuser.clone(), self.userid);
        //         // println!("Enter the ref number of the user you would like to follow:");
        //         // ui.text_edit_singleline(&mut fusernum);
        //         // follow_user(self.userid, fuser.clone());
                                        
                
        //   }
        let mut userlist: Vec<publicUser> = Vec::new();
        ui.heading("Follow a Voicer User");
        ui.add_space(10.0);
        // Group the contents of the login page
        ui.group(|ui| {
            // Calculate available width for each column
            let column_width = ui.available_width();
            ui.horizontal(|ui| {
                ui.label("Enter Username: ");
                let current_width = ui.available_width();
                ui.add_space(70.0-(column_width-current_width)); // Add spacing between the label and the text edit
                ui.text_edit_singleline(&mut self.followuser);
            });
            if ui.button("Search").clicked() 
            {   
                let user2=self.followuser.clone();
                let userid2= self.userid.clone();
                let rt= Runtime::new().unwrap();
                    let (userlistr) = rt.block_on( async move
                        {
                            let response = tokio::spawn
                            ( async move
                                {
                                    let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                    let userlistr=find_users_by_names(user_collection, & user2, userid2).await;
                                    userlistr
                                }
                            ).await.unwrap();
                            response
                        });

                self.userslist=userlistr;
                self.current_page=Page::FollowerPost;
                         
               
            }

        });
    
        ui.add_space(10.0);
        
    }
        // Function to show the Shared Files page UI
    fn shared_files_page_2(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {

            ui.label(format!("User ID: {}", self.userslist));
            
            if ui.button("Follow").clicked() {
                let mut myuser=self.userid;
                let mut myfol=self.userslist;
                let rt= Runtime::new().unwrap();
                    let (userlistr) = rt.block_on( async move
                        {
                            let response = tokio::spawn
                            ( async move
                                {
                                    let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                    db_config::follow(user_collection.clone(), myuser, myfol).await;

                                }
                            ).await.unwrap();
                            response
                        });
                ui.label(format!("Successfull"));
                self.current_page=Page::Home;
            }
            if ui.button("Back").clicked() {
                self.current_page = Page::Home;
            }
                
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
                Page::MyTweet => {
                    self.tweet_page(ctx, ui);
                }
                Page::Follow => {
                    self.FollowPage(ctx, ui);
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
use rodio::{Decoder};

// Function to play the audio
fn play_audio(filename: &str) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open(filename).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    // Play the audio
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


fn delete_wav_files() -> io::Result<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;

    // Read the directory entries
    let entries = fs::read_dir(current_dir)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            if let Some(extension) = file_path.extension() {
                if extension == "wav" {
                    // Delete the file
                    fs::remove_file(file_path)?;
                }
            }
        }
    }

    Ok(())
}

