use std::{fs::{self, File}, path::Path, vec};
use chrono::{DateTime, Utc, TimeZone};
use eframe::{run_native, epi::App, egui::{self}};
use self::egui::color;
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


enum Page {
    Signup,
    Login,
    Home,
    MyTweet,
    Follow,
    FollowerPost,
    Conversation
}


pub struct Gui {
    current_page: Page,
    error_message: Option<String>,
    userslist: ObjectId,
    voicenote_vec: Option<Vec<db_config::VoiceNote>>,
    username: String,
    followuser: String,
    password: String,
    confirm_pass: String,
    email: String,
    userid: ObjectId,
    conversation: Option<db_config::conversation>,
    theme: Theme,
}

// Define an enum to represent the current theme/mode
enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light // Set the default theme/mode to dark
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
            confirm_pass: String::new(),
            theme: Theme::default(),
            voicenote_vec: None,
            userslist : ObjectId::new(),
            userid: ObjectId::new(),
            conversation: None,
        }
    }


// Function to show the signup page UI
fn signup_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
    let mut password_visible = true;
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.heading("Sign up");
        ui.add_space(10.0);
    });
    ui.vertical_centered(|ui| {
        // Group the contents of the signup page
        ui.vertical_centered(|ui|{

            ui.add_space(50.0);
            ui.horizontal(|ui|{
                ui.add_space(600.0);
            });
        }
        );});

    ui.vertical_centered(|ui| {
            // Group the contents of the signup page
            ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    // Group the contents of the signup page
                    ui.vertical_centered(|ui|{
            
                        ui.add_space(10.0);
                    }
                    );});
                // Calculate available width for each column
                let column_width = ui.available_width();
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Username: ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width)); // Add spacing between the heading and the buttons
                    ui.text_edit_singleline(&mut self.username);
                });
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Email: ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width)); // Add spacing between the label and the text edit
                    ui.text_edit_singleline(&mut self.email);
                });
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Password:  ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width));
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(password_visible));
                    //ui.text_edit_singleline(&mut self.password);
                });
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Confirm Password:  ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width));
                    ui.add(egui::TextEdit::singleline(&mut self.confirm_pass).password(password_visible));
                    //ui.text_edit_singleline(&mut confirm_pass);
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.add_space(310.0);
                    ui.add(egui::Checkbox::new(&mut password_visible, "Show Password"));
                });
    
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                ui.add_space(550.0);
                if ui.button("Sign up").clicked() {
                    // Handle signup button click
                    if self.password != self.confirm_pass {
                        // Show an error message if the passwords don't match
                        self.error_message = Some("Passwords don't match".to_string()); // Store error message in a variable
                    } else if self.username.is_empty() || self.password.is_empty() || self.email.is_empty() {
                        // Show an error message if either the username or password is empty
                        self.error_message = Some("All fields are required".to_string()); // Store error message in a variable
                    } else {
                        let rt = Runtime::new().unwrap();
                        let username = self.username.clone();
                        let email = self.email.clone();
                        let pass = self.password.clone();
                        let (response) = rt.block_on(async move {
                            let response = tokio::spawn(async move {
                                let (user_collection, voice_note_collection, db, client) =
                                    db_config::connect_to_mongodb().await;
                                let response = db_config::create_user(user_collection, email, pass, username).await;
    
                                response
                            })
                            .await
                            .unwrap();
                            response
                        });
                        self.current_page = Page::Login;
                    }
                }
                if let Some(error_message) = &self.error_message {
                    // Display error message if it exists
                    ui.add(egui::Label::new(error_message).text_color(egui::Color32::RED));
                }});
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    ui.add_space(750.0);
                });
            });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("Already have an account?");
                    if ui.button("Log in").clicked() {
                        // Show the login page if the user clicks the login button
                        self.username.clear();
                        self.password.clear();
                        self.error_message = None;
                        self.current_page = Page::Login;
                    }
                });
                ui.vertical_centered(|ui| {
                    // Group the contents of the signup page
                    ui.vertical_centered(|ui|{
                      ui.add_space(20.0);
                     
                    }
                    );});
            });
        
    }
    // Function to show the login page UI
    fn login_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui|{
            ui.add_space(10.0);
            ui.heading("Login");
            ui.add_space(10.0);
            

        });
        ui.vertical_centered(|ui| {
            // Group the contents of the signup page
            ui.vertical_centered(|ui|{
    
                ui.add_space(50.0);
                ui.horizontal(|ui|{
                    ui.add_space(600.0);
                });
            }
            );});
        
        ui.add(egui::Label::new("Enter  your Voicer account details:").heading());
        ui.add_space(5.0);
        // Group the contents of the login page
        ui.vertical_centered(|ui| {
            // Group the contents of the signup page
            ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    // Group the contents of the signup page
                    ui.vertical_centered(|ui|{
            
                        ui.add_space(10.0);
                    }
                    );});
                // Calculate available width for each column
                let column_width = ui.available_width();
            
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Username: ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width)); // Add spacing between the heading and the buttons
                    ui.text_edit_singleline(&mut self.username);
                });

                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Password :  ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0-(column_width-current_width)); // Add spacing between the heading and the buttons    
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                ui.add_space(558.0);
                    if ui.button("Log in").clicked() {
     
                    let rt= Runtime::new().unwrap();
                    let username= self.username.clone();
                    let pass= self.password.clone();
                    let (response) = rt.block_on( async move
                        {
                            let response = tokio::spawn
                            ( async move
                                {
                                    let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                    let response = db_config::get_user_by_username(user_collection.clone(), username, pass).await;
                                    let user_iddd= response.unwrap()._id.clone();
                                    let response2 = db_config::get_all_voice_ids_from_following(user_collection ,voice_note_collection , user_iddd).await;
            
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
                    });
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.add_space(750.0);
                        ui.label("");
                        
                    });

            });
            
        });

        ui.vertical(|ui|{
            ui.add_space(5.0);
        });


        if let Some(error_message) = &self.error_message {
            // Display error message if it exists
            ui.add(egui::Label::new(error_message).text_color(egui::Color32::RED));
        }
        ui.vertical(|ui|{
                ui.add_space(10.0);
    
    
            });
        ui.horizontal(|ui| {
                ui.label("Don't have an account?");
                if ui.button("Sign up").clicked() {
                    self.error_message = None;
                    self.username.clear();
                    self.password.clear();
                    self.current_page = Page::Signup;
                    self.userslist= ObjectId::new();
                    self.voicenote_vec= None;
                    self.followuser.clear();
                    self.email.clear();
                    self.userid= ObjectId::new();
                    self.conversation= None; 
                }
            });
            
    ;}

    



// Function to show the home page UI
fn home_page(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui) {
    let folder_name = format!("{}", self.userid);
    fs::create_dir_all(&folder_name).unwrap(); 
    ui.heading("Voicer Home Page");
    ui.add_space(10.0);
    ui.label(format!("Welcome, {}!", self.username)); // Display welcome message with user's name

    // Count the number of voicenotes
    let voicenote_count = self.voicenote_vec.clone().unwrap().len();
    ui.label(format!("You have {} voicenotes.", voicenote_count));

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.add_space(300.0);
        if ui.button("▶️ Play All").clicked() {
            // Play all voicenote files
            let directory = ".";
            let files = fs::read_dir(directory).unwrap();
            let mut filenames: Vec<String> = Vec::new(); 

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


            for filename in filenames {
                play_audio(&filename);
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
            self.followuser.clear();
            self.email.clear();
            self.userid= ObjectId::new();
            self.conversation= None;       
            self.error_message = None;
            self.username.clear();
            self.password.clear();
            self.userslist= ObjectId::new();
            self.voicenote_vec= None;

            self.current_page = Page::Login;
        }
    });

    ui.add_space(10.0);

    let mut vec_vc = self.voicenote_vec.clone();
    egui::ScrollArea::auto_sized().show(ui, |ui| {
        let userid = self.userid.clone();
        if let Some(vec_vc) = vec_vc{
            let voice = vec_vc;
            // Display voicenote posts
            for i in 0..voicenote_count {
                let voice_obj = voice[i].clone();
                ui.group(|ui| {
                    ui.label(format!("Voicenote {} by {}",i+1, voice_obj.name));

                    ui.horizontal(|ui| {
                        if ui.button("▶️ Play").clicked() {
                            let filename = voice_obj._id.to_hex()+".wav";
                            let x = play_audio(&filename);
                        }
                    });

                    let time = Utc.timestamp(voice_obj.timestamp.timestamp(), 0);
                    let formatted_time = time.format("%Y-%m-%d %H:%M:%S").to_string();

                    ui.label(format!("Posted on: {}", formatted_time));
                    let mut reaction = db_config::ReactionType::SpeakUp;
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Shut Up").clicked() {
                                reaction = db_config::ReactionType::ShutUp;
                                let rt= Runtime::new().unwrap();
                                let (response) = rt.block_on( async move
                                    {
                                        let response = tokio::spawn
                                        ( async move
                                            {
                                                let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                                db_config::react_to_quote(voice_note_collection, voice_obj._id,userid,reaction.clone()).await;
                                            }
                                        ).await.unwrap();
                                        response
                                    }
                                );
                            }
                            if ui.button("Speak Up").clicked() {
                                reaction = db_config::ReactionType::SpeakUp;
                                let rt= Runtime::new().unwrap();
                                let (response) = rt.block_on( async move
                                    {
                                        let response = tokio::spawn
                                        ( async move
                                            {
                                                let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                                db_config::react_to_quote(voice_note_collection, voice_obj._id,userid,reaction.clone()).await;
                                            }
                                        ).await.unwrap();
                                        response
                                    }
                                );
                            }
                            if ui.button("Conversation").clicked() {
                                let rt= Runtime::new().unwrap();
                                let (response) = rt.block_on( async move
                                    {
                                        let response = tokio::spawn
                                        ( async move
                                            {
                                                let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                                let replies = db_config::create_conversation(voice_note_collection, voice_obj._id).await;
                        
                                                (replies)
                                            }
                                        ).await.unwrap();
                                        response
                                    });
                                self.conversation = Some(response);
                                self.current_page= Page::Conversation;
                            }
                        });
                    });

                    
                });
            }
        }
    });

    ui.add_space(10.0);
}

fn conversation(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui){
    ui.heading("Conversation");
    ui.add_space(10.0);
    let folder_name = format!("{}", self.userid);
    fs::create_dir_all(&folder_name).unwrap();
    let mut file_name = ObjectId::new();
    let directory = format!("{}/{}.wav", folder_name, file_name.to_hex());
    let mut reply = self.conversation.clone().unwrap();
    let userid = self.userid.clone();
    
    if ui.button("Back").clicked() {
        self.current_page = Page::Home;
    }

    ui.add_space(10.0);

    if ui.button("Add reply").clicked() {
        match ac::record(None) {
            Ok(clip) => {
                match clip.export(format!("{}" , directory).as_str()) {
                    Ok(_) => {
                        println!("Successfully saved!");
                    }
                    Err(err) => println!("Error {}", err),
                }
            }
            Err(err) => println!("Error {}", err),
        }
        let rt= Runtime::new().unwrap();
        let response = rt.block_on( async move
            {
                let response = tokio::spawn
                ( async move
                    {
                        let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                        let data = db_config::convert_audio_to_vec(&directory).await;
                        db_config::create_comment(voice_note_collection.clone(),user_collection, userid, reply.v_id.to_hex() , file_name, data).await;
                        match fs::remove_dir_all(&(Path::new(&folder_name))) {
                            Ok(_) => println!("Directory deleted successfully"),
                            Err(err) => println!("Error deleting directory: {}", err),
                        }
                        let replies = db_config::create_conversation(voice_note_collection, reply.v_id).await;
                
                        (replies)
                    }
                ).await.unwrap();
                response
            });
        self.conversation = Some(response);
        self.current_page= Page::Conversation;   
    }

    let mut reply_count = reply.replies.len();

    egui::ScrollArea::auto_sized().show(ui, |ui| {
    let voice_obj = reply.clone().replies;
    // Display voicenote posts
    for i in 0..reply_count {
        let voice = voice_obj[i].clone();
        ui.group(|ui| {
            ui.label(format!("Reply {} by {}",i+1, voice.user_id.1));
            // Add content for each voicenote post here

            // Play single voicenote button
            ui.horizontal(|ui| {
                if ui.button("▶️ Play").clicked() {
                    let filename = voice._id.to_hex()+".wav";
                    let x = play_audio(&filename);
                }
            });

            let mut reaction = db_config::ReactionType::SpeakUp;
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Shut Up").clicked() {
                        reaction = db_config::ReactionType::ShutUp;
                        let rt= Runtime::new().unwrap();
                        let (response) = rt.block_on( async move
                            {
                                let response = tokio::spawn
                                ( async move
                                    {
                                        let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                        db_config::react_to_quote(voice_note_collection, voice._id,userid,reaction.clone()).await;
                                    }
                                ).await.unwrap();
                                response
                            }
                        );
                    }
                    if ui.button("Speak Up").clicked() {
                        reaction = db_config::ReactionType::SpeakUp;
                        let rt= Runtime::new().unwrap();
                        let (response) = rt.block_on( async move
                            {
                                let response = tokio::spawn
                                ( async move
                                    {
                                        let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                                        db_config::react_to_quote(voice_note_collection, voice._id,userid,reaction.clone()).await;
                                    }
                                ).await.unwrap();
                                response
                            }
                        );
                    }
                });
            });

            
        });
    }})
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
    });
    
    // Record button
    if ui.button("Record").clicked() {
        match ac::record(None) {
            Ok(clip) => {
                match clip.export(format!("{}" , directory).as_str()) {
                    Ok(_) => {
                        println!("Successfully saved!");
                    }
                    Err(err) => println!("Error {}", err),
                }
            }
            Err(err) => println!("Error {}", err),
        }
        let userid = self.userid;
        let rt= Runtime::new().unwrap();
        let response = rt.block_on( async move
            {
                let response = tokio::spawn
                ( async move
                    {
                        let (user_collection, voice_note_collection, db, client) = db_config::connect_to_mongodb().await;
                        println!("{userid}");
                        let data = db_config::convert_audio_to_vec(&directory).await;
                        db_config::create_post(voice_note_collection,user_collection, userid, data, file_name).await;
                        match fs::remove_dir_all(&(Path::new(&folder_name))) {
                            Ok(_) => println!("Directory deleted successfully"),
                            Err(err) => println!("Error deleting directory: {}", err),
                        }
                        is_saved = true;
                    }
                ).await.unwrap();
                response
            });        
    }
    if is_saved {
        ui.label("Voicenote saved successfully!");
    }

    // Display success message if voicenote is saved

    if ui.button("Back").clicked() {
        self.current_page = Page::Home;
    }


    ui.add_space(10.0);

    }
    // Function to show the Shared Files page UI
fn FollowPage(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {
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
                ui.add_space(100.0-(column_width-current_width)); // Add spacing between the label and the text edit
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
    fn follow_user_page(&mut self, _ctx: &egui::CtxRef, ui: &mut egui::Ui) {

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
                    self.follow_user_page(ctx, ui);
                }
                Page::Conversation => {
                    self.conversation(ctx, ui);
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

