mod db_config;

use db_config::connect_to_mongodb;
use record_audio::audio_clip::AudioClip as ac;
use tokio;
use std::io;
use mongodb::{Collection, bson::oid::ObjectId};

#[tokio::main]
async fn main() {
    let user_collection:Collection<db_config::Users> = connect_to_mongodb().await;
    let mut input = String::new();

    


    let mut user_id=ObjectId::new();
    let mut authenticated = false;

    while(!authenticated){
        println!("Do you want to login or signup? (l/s): ");
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                if input.trim() == "s" {
                    user_id = db_config::sign_up(user_collection.clone()).await;
                    authenticated= true;
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

}