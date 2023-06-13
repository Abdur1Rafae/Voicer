use std::{fs::{self, File}, path::Path, vec};
use chrono::{DateTime, Utc, TimeZone};
pub use eframe::{run_native, egui, App};
use egui::{Ui, Color32};
use crate::backend::{self, Users, publicUser, get_user_by_username, find_users_by_names};
use mongodb::{Client, Collection  , Database};
use mongodb::bson::{self,oid::ObjectId};
use tokio::{io, time::Instant, runtime::Runtime};
use tokio;
use std::io::BufReader;
use rodio::{OutputStream, Sink, source::Buffered};
use std::{process::Command};
use backend::{connect_to_mongodb, VoiceNote};
use record_audio::audio_clip::AudioClip as ac;
use egui::TextStyle;
use egui::RichText;
use egui::widgets::Button;

pub struct Gui {
    current_page: Page,
    error_message: Option<String>,
    userslist: Option<backend::publicUser>,
    voicenote_vec: Option<Vec<backend::VoiceNote>>,
    username: String,
    followuser: String,
    password: String,
    confirm_pass: String,
    email: String,
    user: Option<backend::Users>,
    conversation: Option<backend::conversation>,
    theme: Theme,
    following: Option<Vec<backend::publicUser>>,
    followers: Option<Vec<backend::publicUser>>,
    window_style: egui::Style,
}

enum Page {
    Signup,
    Login,
    Home,
    MyTweet,
    Follow,
    FollowerProfile,
    Conversation,
    UserProfile,
    Following,
    Followers
}

enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}


impl Gui {
    fn toggle_theme(&mut self, ctx: &egui::Context) {
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
            userslist : None,
            user: None,
            conversation: None,
            following: None,
            followers: None,
            window_style: egui::Style::default(),  
        }
    }


fn signup_page(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
    let mut password_visible = true;
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.heading("Sign up");

        ui.add_space(10.0);
    });
    ui.vertical_centered(|ui| {

        ui.vertical_centered(|ui|{

            ui.add_space(50.0);
            ui.horizontal(|ui|{
                ui.add_space(600.0);
            });
        }
        );});


    ui.vertical_centered(|ui| {

            ui.group(|ui| {
                ui.vertical_centered(|ui| {

                    ui.vertical_centered(|ui|{
            
                        ui.add_space(10.0);
                    }
                    );});

                let column_width = ui.available_width();
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Username: ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width)); 
                    ui.text_edit_singleline(&mut self.username);
                });
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Email: ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width)); 
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
                        let runtime = Runtime::new().unwrap();
                        let username = self.username.clone();
                        let email = self.email.clone();
                        let pass = self.password.clone();
                        let (response) = runtime.block_on(async move {
                            let response = tokio::spawn(async move {
                                let (user_collection, voice_note_collection, db, client) =
                                    backend::connect_to_mongodb().await;
                                let response = backend::create_user(user_collection, email, pass, username).await;
    
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
                    ui.add(egui::Label::new(error_message));
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
                    });
                });
            });
        
    }
    // Function to show the login page UI
    fn login_page(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui|{
            ui.add_space(10.0);
            ui.heading("Login");
            ui.label(RichText::new(("Test")).color(egui::Color32::DARK_RED));
            ui.add_space(10.0);
            ui.add(egui::Button::new("Test").fill(Color32::RED)).clicked();
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
        
        ui.add(egui::Label::new("Enter  your Voicer account details:"));
        ui.add_space(5.0);
        ui.vertical_centered(|ui| {
            ui.group(|ui| {
                ui.vertical_centered(|ui| {
                    ui.vertical_centered(|ui|{
            
                        ui.add_space(10.0);
                    }
                    );
                });
                let column_width = ui.available_width();
            
                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Username: ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0 - (column_width - current_width)); 
                    ui.text_edit_singleline(&mut self.username);
                });

                ui.horizontal(|ui| {
                    ui.add_space(200.0);
                    ui.label("Password :  ");
                    let current_width = ui.available_width();
                    ui.add_space(310.0-(column_width-current_width));   
                    ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.add_space(558.0);
                    if ui.button(RichText::new("Log in")).clicked() {
        
                        let runtime= Runtime::new().unwrap();
                        let username= self.username.clone();
                        let pass= self.password.clone();
                        let (response) = runtime.block_on( async move
                            {
                                let response = tokio::spawn
                                ( async move
                                    {
                                        let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                        let response = backend::get_user_by_username(user_collection.clone(), username, pass).await;
                                        let user_iddd= response.clone().unwrap()._id;
                                        let response2 = backend::get_all_voice_ids_from_following(user_collection ,voice_note_collection , user_iddd).await;
                
                                        (response,response2)
                                    }
                                ).await.unwrap();
                                response
                            });

                        self.user = response.0.clone();
                        self.voicenote_vec = Some(response.1.clone());
                        //println!("login successful : {:?}",response.0.clone() );
                        println!("LOGIN SUCCESSFUL");
                        self.current_page = Page::Home;
                    }
                });
            });
            
        });

        ui.vertical(|ui|{
            ui.add_space(5.0);
        });

        if let Some(error_message) = &self.error_message {
            ui.add(egui::Label::new(error_message));
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
                self.userslist= None;
                self.voicenote_vec= None;
                self.followuser.clear();
                self.email.clear();
                self.user= None;
                self.conversation= None; 
            }
        });
            
    }

    

fn home_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.heading("Voicer Home Page");
    ui.add_space(10.0);
    ui.label(format!("Welcome, {}!", self.username));

    // Count the number of voicenotes
    let voicenote_count = self.voicenote_vec.clone().unwrap().len();
    ui.label(format!("You have {} voicenotes.", voicenote_count));

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.add_space(420.0);
        if ui.button("▶️ Play All").clicked() {
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
            self.current_page=Page::MyTweet;                        
        }
        if ui.button("➹ Follow").clicked() {
            self.current_page=Page::Follow;                        
        }
        if ui.button("Theme").clicked() {
            self.toggle_theme(ctx);
        }

        if ui.button("Profile").clicked() {
            let user = self.user.clone().unwrap();
            let runtime = Runtime::new().unwrap();
            let mut delete:Vec<ObjectId> = runtime.block_on( async move
                {
                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                    let mut delete:Vec<ObjectId> = Vec::new();
                    for quote in user.voice_notes {
                        let response = backend::download_voice_notes(voice_note_collection.clone(), quote).await;
                        if response == false {
                            delete.push(quote);
                        }
                    }
                    delete
                }
            );
            for quote in &mut delete{
                self.user.as_mut().unwrap().voice_notes.retain(|voiceid| (voiceid.to_hex())!=(quote.to_hex()));
            }
            self.current_page = Page::UserProfile;
        }


        if ui.button("Logout").clicked() {
            if let Err(err) = delete_wav_files() {
                eprintln!("Error deleting .wav files: {}", err);
            } else {
                println!("Successfully deleted .wav files");
            }  
            self.followuser.clear();
            self.email.clear();
            self.user= None;
            self.conversation= None;       
            self.error_message = None;
            self.username.clear();
            self.password.clear();
            self.userslist= None;
            self.voicenote_vec= None;
            self.current_page = Page::Login;
        }
    });

    ui.add_space(10.0);
    
    

    let mut vec_vc = self.voicenote_vec.clone();
    egui::ScrollArea::vertical().show(ui, |ui| {
        let userid = self.user.clone().unwrap()._id;
        if let Some(vec_vc) = vec_vc{
            let voice = vec_vc;
            for i in 0..voicenote_count {
                let voice_obj = voice[i].clone();
                ui.horizontal(|ui|{
                    ui.add_space(350.0);
                    ui.group(|ui| {
                        ui.horizontal(|ui|{
                            ui.add_space(150.0);
                            ui.vertical(|ui|{
                                ui.label(format!("Voicenote {} by {}",i+1, voice_obj.name));
                                ui.horizontal(|ui| {
                                    ui.add_space(75.0);
                                    if ui.add(egui::Button::new("▶️ Play").fill(Color32::TRANSPARENT)).clicked() {
                                        let filename = voice_obj._id.to_hex()+".wav";
                                        let x = play_audio(&filename);
                                    }
                                });
            
                                let time = Utc.timestamp(voice_obj.timestamp.timestamp(), 0);
                                let formatted_time = time.format("%Y-%m-%d %H:%M:%S").to_string();
            
                                ui.label(format!("Posted on: {}", formatted_time));
                                let mut reaction = backend::ReactionType::SpeakUp;
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui.add(egui::Button::new(RichText::new(("Shut Up")).color(egui::Color32::WHITE)).fill(Color32::LIGHT_RED)).clicked() {
                                            reaction = backend::ReactionType::ShutUp;
                                            let runtimet= Runtime::new().unwrap();
                                            let (response) = runtimet.block_on( async move
                                                {
                                                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                                    backend::react_to_quote(voice_note_collection, voice_obj._id,userid,reaction.clone()).await
                                                }
                                            );
                                        }
                                        if ui.add(egui::Button::new(RichText::new(("Speak Up")).color(egui::Color32::WHITE)).fill(Color32::LIGHT_GREEN)).clicked() {
                                            reaction = backend::ReactionType::SpeakUp;
                                            let runtime= Runtime::new().unwrap();
                                            let (response) = runtime.block_on( async move
                                                {
                                                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                                    backend::react_to_quote(voice_note_collection, voice_obj._id,userid,reaction.clone()).await;
                                                }
                                            );
                                        }
                                        if ui.add(egui::Button::new(RichText::new(("Reply")).color(egui::Color32::WHITE)).fill(Color32::LIGHT_BLUE)).clicked() {
                                            let runtime= Runtime::new().unwrap();
                                            let (response) = runtime.block_on( async move
                                                {
                                                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                                    let replies = backend::create_conversation(voice_note_collection, voice_obj._id).await;
                            
                                                    (replies)
                                                });
                                            self.conversation = Some(response);
                                            self.current_page= Page::Conversation;
                                        }
                                    });
                                });
                            });
                            ui.add_space(150.0);
                        });
                        
                    });
                });
            }
        }
    });

    ui.add_space(10.0);
}

fn conversation(&mut self, ctx: &egui::Context, ui: &mut egui::Ui){
    ui.heading("Conversation");
    ui.add_space(10.0);
    let folder_name = format!("{}", self.user.clone().unwrap()._id);
    fs::create_dir_all(&folder_name).unwrap();
    let mut file_name = ObjectId::new();
    let directory = format!("{}/{}.wav", folder_name, file_name.to_hex());
    let mut reply = self.conversation.clone().unwrap();
    let userid = self.user.clone().unwrap()._id;;
    
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
        let runtime= Runtime::new().unwrap();
        let response = runtime.block_on( async move
            {
                let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                let data = backend::convert_audio_to_vec(&directory).await;
                backend::create_comment(voice_note_collection.clone(),user_collection, userid, reply.v_id.to_hex() , file_name, data).await;
                match fs::remove_dir_all(&(Path::new(&folder_name))) {
                    Ok(_) => println!("Directory deleted successfully"),
                    Err(err) => println!("Error deleting directory: {}", err),
                }
                let replies = backend::create_conversation(voice_note_collection, reply.v_id).await;
        
                (replies)
            });
        self.conversation = Some(response);
        self.current_page= Page::Conversation;   
    }

    let mut reply_count = reply.replies.len();

    egui::ScrollArea::vertical().show(ui, |ui| {
        let voice_obj = reply.clone().replies;
        // Display voicenote posts
        for i in 0..reply_count {
            let voice = voice_obj[i].clone();
            ui.group(|ui| {
                ui.label(format!("Reply {} by {}",i+1, voice.user_id.1));

                ui.horizontal(|ui| {
                    if ui.button("▶️ Play").clicked() {
                        let filename = voice._id.to_hex()+".wav";
                        let x = play_audio(&filename);
                    }
                });

                let mut reaction = backend::ReactionType::SpeakUp;
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("Shut Up").fill(Color32::LIGHT_RED)).clicked() {
                            reaction = backend::ReactionType::ShutUp;
                            let runtime= Runtime::new().unwrap();
                            let (response) = runtime.block_on( async move
                                {
                                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                    backend::react_to_quote(voice_note_collection, voice._id,userid,reaction.clone()).await;
                                }
                            );
                        }
                        if ui.add(egui::Button::new("Shut Up").fill(Color32::LIGHT_GREEN)).clicked() {
                            reaction = backend::ReactionType::SpeakUp;
                            let runtime= Runtime::new().unwrap();
                            let (response) = runtime.block_on( async move
                                {
                                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                    backend::react_to_quote(voice_note_collection, voice._id,userid,reaction.clone()).await;
                                }
                            );
                        }
                    });
                });

                
            });
        }
    });
}

fn tweet_page(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
    let column_width = ui.available_width();
    let folder_name = format!("{}", self.user.clone().unwrap()._id);
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
        let userid =self.user.clone().unwrap()._id;;
        let runtime= Runtime::new().unwrap();
        let response = runtime.block_on( async move
            {
                let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                let data = backend::convert_audio_to_vec(&directory).await;
                backend::create_post(voice_note_collection,user_collection, userid, data, file_name).await;
                match fs::remove_dir_all(&(Path::new(&folder_name))) {
                    Ok(_) => println!("Directory deleted successfully"),
                    Err(err) => println!("Error deleting directory: {}", err),
                }
                is_saved = true;
            });        
    }
    if is_saved {
        ui.label("Voicenote saved successfully!");
    }

    if ui.button("Back").clicked() {
        self.current_page = Page::Home;
    }

    ui.add_space(10.0);

}
    // Function to show the Shared Files page UI
fn FollowPage(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
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
                let userid2= self.user.clone().unwrap()._id;
                let runtime= Runtime::new().unwrap();
                    let (userlistr) = runtime.block_on( async move
                        {
                            let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                            let userlistr=find_users_by_names(user_collection, & user2, userid2).await;
                            userlistr
                        });
                self.userslist=Some(userlistr);
                self.current_page=Page::FollowerProfile;
            }
        });
        ui.add_space(10.0);
    }

    fn follow_user_page(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {

        ui.label(format!("User Profile"));
        ui.add_space(15.0);
        let user = self.userslist.clone().unwrap();
        ui.label(format!("Name:\t\t\t{}", user.name));
        ui.label(format!("Username:\t{}", user.username));
        ui.label(format!("Bio:       {}", user.description));
        ui.label(format!("Quotes:\t\t{}", user.voice_notes.len()));
        ui.label(format!("Followers: {}", user.followers.len()));
        ui.label(format!("Following: {}", user.following.len()));
        
        if ui.button("Follow").clicked() {
            let mut myuser=self.user.clone().unwrap()._id;
            let mut myfol=self.userslist.clone().unwrap()._id;
            let runtime= Runtime::new().unwrap();
                let (userlistr) = runtime.block_on( async move
                    {
                        let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                        backend::follow(user_collection.clone(), myuser, myfol).await;

                    });
            ui.label(format!("Successfull"));
            self.current_page=Page::Home;
        }
        if ui.button("Back").clicked() {
            self.current_page = Page::Home;
        } 
    }


    fn user_profile(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui){
        let mut your_info = self.user.clone().unwrap();
        ui.vertical_centered(|ui|{
            ui.add_space(10.0);
            ui.heading("Your Profile");
        });

        ui.label(format!("Name:\t\t\t{}", your_info.name));
        ui.label(format!("Username:\t{}", your_info.username));
        ui.horizontal(|ui|{
            ui.label("Bio: ");
            let current_width = ui.available_width();
            ui.text_edit_singleline(&mut your_info.description);
            if ui.button("Update Bio").clicked() {
                let runtime= Runtime::new().unwrap();
                let (userlistr) = runtime.block_on( async move
                    {
                        let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                        backend::update_description_by_username(user_collection, &your_info.username, &your_info.description).await;
                    });
                    ui.label(format!("Successfull"));
                    self.current_page=Page::Home;
            }
        });
        let followers_count = your_info.followers.len();
        let following_count = your_info.following.len();
        
        if ui.add(egui::Button::new(format!("Followers: {}", followers_count))).clicked() {
            let runtime= Runtime::new().unwrap();
            let (userlistr) = runtime.block_on( async move
                {
                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                    backend::get_all_followers_profile(user_collection, your_info._id).await
                });
            self.followers = Some(userlistr);
            self.current_page=Page::Followers;
        };

        if ui.add(egui::Button::new(format!("Following: {}", following_count))).clicked() {
            let runtime= Runtime::new().unwrap();
            let (userlistr) = runtime.block_on( async move
                {
                    let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                    backend::get_all_following_profile(user_collection, your_info._id).await
                });
            self.following = Some(userlistr);
            self.current_page=Page::Following;
        };
        
        let mut quotes_count = your_info.voice_notes.len();


        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..quotes_count {
                ui.group(|ui| {
                    ui.label(format!("Quote {}",i+1));

                    ui.horizontal(|ui| {
                        if ui.button("▶️ Play").clicked() {
                            let filename = your_info.voice_notes[i].to_hex()+".wav";
                            let x = play_audio(&filename);
                        }
                        let post = your_info.voice_notes[i].clone();
                        if ui.add(egui::Button::new(RichText::new(("Delete")).color(egui::Color32::WHITE)).fill(Color32::RED)).clicked() {
                            let runtime= Runtime::new().unwrap();
                            let (userlistr) = runtime.block_on( async move
                            {
                                let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                                backend::delete_post(voice_note_collection, user_collection, post, your_info._id).await
                            });
                        }
                    });
                });
            }
        });

        if ui.button("HOME").clicked() {
            self.current_page= Page::Home;
        }

    }

    fn following_profiles_display(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label(format!("You Follow: "));
        let mut followingList = self.following.clone().unwrap();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..followingList.len(){
                ui.add_space(15.0);
                ui.group(|ui|{
                    let user = followingList[i].clone();
                    ui.label(format!("Name:\t\t\t{}", user.name));
                    ui.label(format!("Username:\t{}", user.username));
                    ui.label(format!("Bio:       {}", user.description));
                    ui.label(format!("Quotes:\t\t{}", user.voice_notes.len()));
                    ui.label(format!("Followers: {}", user.followers.len()));
                    ui.label(format!("Following: {}", user.following.len()));
                    
                    if ui.add(egui::Button::new(RichText::new(("Unfollow")).color(egui::Color32::RED))).clicked() {
                        let mut myuser=self.user.clone().unwrap()._id;
                        let mut following=user._id.clone();
                        let runtime= Runtime::new().unwrap();
                        let (userlistr) = runtime.block_on( async move
                        {
                            let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                            backend::unfollow(user_collection.clone(), myuser, following).await
                        });
                        self.following = Some(userlistr);
                        ui.label(format!("Successfull"));
                    }
                });
            }
        });
        if ui.button("Back").clicked() {
            self.current_page = Page::Home;
        } 
    }

    fn followers_profiles_display(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.label(format!("You are followed by: "));
        let mut followerList = self.followers.clone().unwrap();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..followerList.len(){
                ui.add_space(15.0);
                ui.group(|ui|{
                    let user = followerList[i].clone();
                    ui.label(format!("Name:\t\t\t{}", user.name));
                    ui.label(format!("Username:\t{}", user.username));
                    ui.label(format!("Bio:       {}", user.description));
                    ui.label(format!("Quotes:\t\t{}", user.voice_notes.len()));
                    ui.label(format!("Followers: {}", user.followers.len()));
                    ui.label(format!("Following: {}", user.following.len()));
                    
                    if ui.add(egui::Button::new(RichText::new(("Remove")).color(egui::Color32::WHITE)).fill(Color32::RED)).clicked() {
                        let mut myuser=self.user.clone().unwrap()._id;
                        let mut follower=user._id.clone();
                        let runtime= Runtime::new().unwrap();
                        let (userlistr) = runtime.block_on( async move
                        {
                            let (user_collection, voice_note_collection, db, client) = backend::connect_to_mongodb().await;
                            backend::remove_follower(user_collection.clone(), myuser, follower).await
                        });
                        self.following = Some(userlistr);
                        ui.label(format!("Successfull"));
                    }
                });
            }
        });
        if ui.button("Back").clicked() {
            self.current_page = Page::Home;
        } 
    }
}
    


impl App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.window_style.visuals.override_text_color = Some(egui::Color32::from_rgb(200, 200, 200));            
            
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
                Page::FollowerProfile => {
                    self.follow_user_page(ctx, ui);
                },
                Page::Conversation => {
                    self.conversation(ctx, ui);
                },
                Page::UserProfile => {
                    self.user_profile(ctx, ui);
                },
                Page::Following => {
                    self.following_profiles_display(ctx, ui);
                },
                Page::Followers => {
                    self.followers_profiles_display(ctx, ui);
                }
            }
        });
    }

}


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

use rodio::{Decoder};


fn play_audio(filename: &str) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = File::open(filename).unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    sink.append(source);
    sink.sleep_until_end();
}


fn pause_audio(sink: &Sink) {
    sink.pause();
}


fn stop_audio(sink: &Sink) {
    sink.stop();
}


fn delete_wav_files() -> io::Result<()> {
    let current_dir = std::env::current_dir()?;

    let entries = fs::read_dir(current_dir)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let file_path = entry.path();
            if let Some(extension) = file_path.extension() {
                if extension == "wav" {
                    fs::remove_file(file_path)?;
                }
            }
        }
    }

    Ok(())
}

