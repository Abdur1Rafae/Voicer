mod db_config;
use std::{fs, process::Command};
use inquire::Text;

use db_config::{connect_to_mongodb, VoiceNote};
use record_audio::audio_clip::AudioClip as ac;
use tokio;
use std::io;
use mongodb::{Collection, bson::oid::ObjectId};
use std::path::Path;
use rodio::Source;
use std::io::BufReader;

#[tokio::main]

async fn main() {
    let (user_collection, voice_note_collection, db, client) = connect_to_mongodb().await;

    let mut input = String::new();

    let mut user_id=ObjectId::new();
    
    let mut authenticated = false;

    while !authenticated{
        println!("Welcome to Voicer, the decentralized voice message Social Platform");
        let login_choice = Text::new("To Login type L or type S to Sign up")
            .with_help_message("Just press enter for quick login, or S for signup")
            .with_default("L")
            .prompt()
            .unwrap();

        if login_choice == ("S") || login_choice == ("s") {
            println!("Signup page");
            user_id = db_config::sign_up_scli(user_collection.clone()).await;
            if user_id != ObjectId::parse_str("f0f0f0f0f0f0f0f0f0f0f0f0").unwrap() {
                authenticated = true;
            }
        } else if login_choice == ("L") || login_choice == ("l") {
            let res_user = db_config::login_scli(user_collection.clone()).await;
            match res_user {
                Some(user) => {
                    user_id = user._id;
                    authenticated = true;
                }
                None => {
                    // Handle case where login is unsuccessful
                    println!("\n\n")
                }
            }
        } else {
            println!("Invalid choice!");
        };
    }
   
    println!("User authenticated with ID: {:?}", user_id);
    println!("This is your unique recognizer for Voicer.");
    let folder_name = format!("{}", user_id);
    fs::create_dir_all(&folder_name).unwrap(); 
    println!("\n\n This is your Voicer");
    println!("What would you like to do?");
    println!("Enter t to Tweet your voicenote");
    println!("Enter f to follow someone");
    println!("Enter c to comment on a tweet");
    println!("Enter rr to retweet a tweet");
    println!("Enter h to go to home page");
    println!("Enter q to quit");

    loop {
    let user_choice = Text::new("What would you like to do? Press Enter to directly go to homepage!")
            .with_default("h")
            .prompt()
            .unwrap();
            
            match user_choice.as_str() {
                "quit" => {
                    break;
                }
                "t" => {
                    let mut file_name = ObjectId::new();
                    let directory = format!("{}/{}.wav", folder_name, file_name.to_hex());
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
                    let data = db_config::convert_audio_to_vec(&directory).await;
                    db_config::create_post(voice_note_collection.clone(), user_collection.clone(), user_id, data, file_name).await;
                    match fs::remove_dir_all(&(Path::new(&folder_name))) {
                        Ok(_) => println!("Directory deleted successfully"),
                        Err(err) => println!("Error deleting directory: {}", err),
                    }
                }
                "f" => {
                    println!("Enter the name");
                    input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            input = input.trim().to_string();
                            let mut userlist = db_config::find_users_by_names(user_collection.clone(), &input, user_id).await;
                            println!("{:#?}", userlist);
                            println!("Enter the ref number of the user you would like to follow:");
                            input = String::new();
                            match io::stdin().read_line(&mut input) {
                                Ok(_) => {
                                    input = input.trim().to_string();
                                    let refNo = input.parse::<i32>().unwrap();
                                    let mut f_user_id = ObjectId::new();
                                    for item in userlist {
                                        if item.refNo == refNo {
                                            f_user_id = item._id;
                                            break;
                                        }
                                    }
                                    db_config::follow(user_collection.clone(), user_id, f_user_id).await;
                                }
                                Err(err) => {
                                    println!("Error {}", err);
                                }
                            } 
                        }
                        Err(err) => {
                            println!("Error {}", err);
                        }
                    }
                }
                "c" => {
                    println!("Enter the voice note id");
                    input = String::new();
            
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            input = input.trim().to_string();
                            let mut file_name = ObjectId::new();
                            let directory = format!("{}/{}.wav", folder_name, file_name.to_hex());
                            match ac::record(None) {
                                Ok(clip) => {
                                    match clip.export(format!("{}" , directory).as_str()) {
                                        Ok(_) => {
                                            println!("Reply saved!");
                                        }
                                        Err(err) => println!("Error {}", err),
                                    }
                                }
                                Err(err) => println!("Error {}", err),
                            }
                            let data = db_config::convert_audio_to_vec(&directory).await;
                            db_config::create_comment(voice_note_collection.clone(), user_collection.clone(), user_id, input, file_name, data).await;
                            match fs::remove_dir_all(&directory) {
                                Ok(_) => println!("Directory deleted successfully"),
                                Err(err) => println!("Error deleting directory: {}", err),
                            }
                        }
                        Err(err) => {
                            println!("Error {}", err);
                        }
                    }
                }
                "rr" => {
                    println!("Enter the voice note id");
                    input = String::new();
                
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            input = input.trim().to_string();
                            let v_id = ObjectId::parse_str(input.clone()).unwrap();
                            input = String::new();
                
                            println!("Enter 0 to react SHUT UP, or 1 to react SPEAK UP");
                
                            match io::stdin().read_line(&mut input) {
                                Ok(_) => {
                                    input = input.trim().to_string();
                                    let refNo = input.parse::<i32>().unwrap();
                                    let reaction: db_config::ReactionType = match refNo {
                                        0 => db_config::ReactionType::ShutUp,
                                        _ => db_config::ReactionType::SpeakUp,
                                    };
                                    db_config::react_to_quote(voice_note_collection.clone(), v_id, user_id, reaction).await;
                                }
                                Err(err) => {
                                    println!("Error {}", err);
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error {}", err);
                        }
                    }
                }
                "h" => {
                    let mut voices = db_config::get_all_voice_ids_from_following(user_collection.clone(), voice_note_collection.clone(), ObjectId::parse_str(&user_id.to_string()).unwrap()).await;
                    db_config::sort_voice_notes_by_timestamp_desc(&mut voices);
                    for i in 0..voices.len() {
                        let mut filename = format!("{}.wav", voices[i]._id.to_string());
                        db_config::convert_vec_to_audio(&filename, voices[i].data.clone()).await;
                    }
                    // Loop to print voice notes
                    loop {
                        println!("Enter the filename of the voice note you want to play (or 'q' to quit):");
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).unwrap();
                        let input = input.trim();

                        if input == "q" {
                            break;
                        }

                        // Check if the file exists
                        let filename = format!("{}.wav", input);
                        let path = Path::new(&filename);

                        if path.exists() {
                            // Print the voice note
                            println!("Printing voice note: {}", input);
                            // Print the voice note
                            // Print the voice note
                            play_audio(&filename);
                            // TODO: Implement the logic to print the voice note here
                        } else {
                            println!("File not found. Please enter a valid filename.");
                        }
                    }

                    
                }
        _ => {
            println!("Invalid choice!");
                }
                }}
}
        
use rodio::{OutputStream, Sink};
use std::fs::File;
            
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
            
