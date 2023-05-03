mod db_config;
use std::fs;

use db_config::{connect_to_mongodb, VoiceNote};
use record_audio::audio_clip::AudioClip as ac;
use tokio;
use std::io;
use mongodb::{Collection, bson::oid::ObjectId};
use std::path::Path;

#[tokio::main]

async fn main() {
    let (user_collection, voice_note_collection, db, client) = connect_to_mongodb().await;
    // let mut voices = db_config::get_all_voice_ids_from_following(user_collection.clone(), voice_note_collection.clone(), ObjectId::parse_str("644f79f076f7ad5bde441e7a".to_string()).unwrap()).await;
    // db_config::sort_voice_notes_by_timestamp_desc(&mut voices);
    // println!("{:?}", voices);
    let mut input = String::new();

    let mut user_id=ObjectId::new();
    let mut authenticated = false;

    while !authenticated{
        println!("Do you want to login or signup? (l/s): ");
        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() == "s" {
                    println!("signup page");
                    user_id = db_config::sign_up(user_collection.clone()).await;
                    if user_id == ObjectId::parse_str("f0f0f0f0f0f0f0f0f0f0f0f0").unwrap() {
                        authenticated = false;
                    }
                    else{
                    authenticated= true;
                }
                } else if input.trim() == "l" {
                    let res_user = db_config::login(user_collection.clone()).await;
                    match res_user {
                        Some(user)=>{
                            user_id = user._id;
                            authenticated = true;
                        }
                        None=>{

                        }
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        };
    }
   
    println!("User authenticated with ID: {:?}", user_id);
    let folder_name = format!("{}", user_id);
    fs::create_dir_all(&folder_name).unwrap(); 

    println!("Would you like to tweet something, follow someone or comment on a tweet? (t , f, c, rr)");

    input = String::new();

    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if input.trim() == "t" {
                let mut file_name= ObjectId::new();
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
                db_config::create_post(voice_note_collection.clone(), user_collection.clone(), user_id, data ,file_name).await;
                match fs::remove_dir_all( &(Path::new(folder_name.as_str()))) {
                    Ok(_) => println!("Directory deleted successfully"),
                    Err(err) => println!("Error deleting directory: {}", err),
                }
            }
            else if input.trim()== "f" {
                println!("Enter the name");
                input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        input = input.trim().to_string();
                        let mut userlist = db_config::find_users_by_names(user_collection.clone(), &input, user_id).await;
                        println!("{:#?}", userlist);
                        println!("Enter the ref number of user you would like to follow:");
                        input = String::new();
                        match io::stdin().read_line(&mut input){
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
                                db_config::follow(user_collection, user_id, f_user_id).await;
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
            else if input.trim() == "c" {
                println!("Enter the voice note id");
                input = String::new();

                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        input = input.trim().to_string();
                        let mut file_name= ObjectId::new();
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
                        match fs::remove_dir_all(directory) {
                            Ok(_) => println!("Directory deleted successfully"),
                            Err(err) => println!("Error deleting directory: {}", err),
                        }
                    }
                    Err(err) => {
                        println!("Error {}", err);
                    }
                }
            }
            else if input.trim() == "rr" {
                println!("Enter the voice note id");
                input = String::new();

                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        input = input.trim().to_string();
                        let v_id= ObjectId::parse_str(input.clone()).unwrap();
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
                                db_config::react_to_quote(voice_note_collection,v_id,user_id,reaction).await;
                            }
                            Err(err) => {
                                println!("Error {}", err);
                            }
                        }
                    },
                    Err(err) => {
                        println!("Error {}", err);
                    }
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    };
}