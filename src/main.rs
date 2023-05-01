mod db_config;
use std::fs;

use db_config::{connect_to_mongodb, VoiceNote};
use record_audio::audio_clip::AudioClip as ac;
use tokio;
use std::io;
use mongodb::{Collection, bson::oid::ObjectId};

#[tokio::main]

async fn main() {
    
    
    //checking find users by names functions
    // let (user_collection, voice_note_collection, db, client) = connect_to_mongodb().await;
    // let mut userlist = db_config::find_users_by_names(user_collection.clone(), "Abdur Rafae").await;
    // println!("{:?}", userlist);
    
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
   

    // Call the create_user function to create a new user
    println!("User created with ID: {:?}", user_id);
    let folder_name = format!("{}", user_id);
    fs::create_dir_all(&folder_name).unwrap(); // use create_dir_all to create the folder and its parent folders if they don't exist

    println!("Would you like to tweet something or follow someone? (t , f)");

    input = String::new();

    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            if input.trim() == "t" {
                let mut file_name= ObjectId::new();
                file_name= db_config::create_post(voice_note_collection.clone(), user_collection.clone(), user_id).await;
                let directory = format!("{}/{}.wav", folder_name, file_name.to_hex());
                match ac::record(None) {
                    Ok(clip) => {
                        println!("Successfully recorded!");
                        match clip.export(format!("{}" , directory).as_str()) {
                            Ok(_) => {
                                println!("Successfully saved!");
                            }
                            Err(err) => println!("Error {}", err),
                        }
                    }
                    Err(err) => println!("Error {}", err),
                }
            }
            else if input.trim()== "f" {
                println!("Enter the name");
                input = String::new();

                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        //let result = db_config::find_users_by_names(user_collection.clone(), input).await;
                        //println!("{:?}", result);
                    }
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