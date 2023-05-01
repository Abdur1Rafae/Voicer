mod db_config;
use std::fs;

use db_config::connect_to_mongodb;
use record_audio::audio_clip::AudioClip as ac;
use tokio;
use std::io;
use mongodb::{Collection, bson::oid::ObjectId};

#[tokio::main]

async fn main() {
    let (user_collection, DB, client) = connect_to_mongodb().await;

    let mut input = String::new();

    let mut user_id=ObjectId::new();
    let mut authenticated = false;

    while(!authenticated){
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
                    match(res_user){
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

    // let updated = db_config::update_description_by_username(user_collection.clone(), "user2", "I am user2 ").await;
    // if updated {
    //     println!("User name updated");
    // } else {
    //     println!("User name not updated");
    // }
    let file_name = "hello.wav";
    match ac::record(None) {
        Ok(clip) => {
            println!("Successfully recorded!");
            match clip.export(format!("{}" , file_name).as_str()) {
                Ok(_) => {
                    println!("Successfully saved as hello.wav");
                    
                    //to play immediately, after ctrl-c
                    //clip.play().unwrap();
                }
                Err(err) => println!("Error {}", err),
            }
        }
        Err(err) => println!("Error {}", err),
    }

  let v_id =  db_config::save_voice_note(user_id).await;
  
  let path = "hello.wav";
    match fs::remove_file(path) {
        Ok(()) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
    }  
  db_config::play_audio("644e4397f424bc54fca3b912").await;
}