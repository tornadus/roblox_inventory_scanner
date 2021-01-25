/*--------------------------------------
|       Roblox Inventory Scanner       |
|             by tornadus              |
|        Last Update: 1/24/2021        |
--------------------------------------*/


//Imports
use reqwest::Client;
use futures::stream::StreamExt;
use serde::{Deserialize};
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, CONTENT_TYPE};
use std::sync::{Arc, Mutex};
use std::io;
use std::io::prelude::{Read, Write};

//Structs
//Rolimon's API
#[derive(Deserialize)]
#[derive(Clone)]
struct RoliApi {
success: bool,
item_count: u64,
items: HashMap<u64, ItemDetails>,
}

//Rolimon's API Item Details
#[derive(Deserialize)]
#[derive(Clone)]
struct ItemDetails {
name: String,
acro: String,
rap: i64,
value: i64,
default_value: i64,
demand: i64,
trend: i64,
projected: i64,
hyped: i64,
rare: i64,
}

//Roblox Ownership API
#[derive(Deserialize)]
struct OwnershipAPI {
_previous_page: Option<String>,
_next_page: Option<String>,
data: Vec<OwnedItem>,
}

//Roblox Ownership API Item Instances
#[derive(Deserialize)]
struct OwnedItem {
_a_type: Option<String>,
id: u64,
name: String,
_instance: Option<u64>,
}

//Functions

fn construct_headers() -> HeaderMap {
    //Creates the headers needed to obtain the Rolimon's API output
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36 Edg/87.0.664.75"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
    headers
}

fn pause() {
    //Stops execution of the program and waits for new line
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press the enter key to continue...").unwrap();
    stdout.flush().unwrap();

    let _ = stdin.read(&mut [0u8]).unwrap();
    let _ = stdin.read(&mut [0u8]).unwrap();
}

//Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting tornadus' inventory scanner v0.1");
    println!("Please enter a user id:");
    let uid: i32 = text_io::read!(); //Get user input for user id
    println!("Scanning user id {}", uid);
    let client = Client::builder().build()?; //HTTP client
    
    //Get Rolimon's API
    let req = client.get("https://www.rolimons.com/itemapi/itemdetails")
        .headers( construct_headers() )
        .send()
        .await?;



    let roli_api: RoliApi = req.json().await?; //Imports the Rolimon's API json as a RoliApi instance
    let mut ids: Vec<u64> = Vec::new(); //Holds item ids gathered from the Rolimon's API. Used for creating the HTTP request URLs.

    let roli_items = roli_api.items.clone(); //Used later on when we need a copy of the Rolimon's item DB

    //Iterate through items and push ids to vector
    for (id, _value) in roli_api.items {
        ids.push(id);
    }
    let found_ids: Vec<OwnershipAPI> = Vec::new(); //Used below
    let found = Arc::new(Mutex::new(found_ids)); //Gathering owned items from the asynchronous HTTP calls

    let fetches = futures::stream::iter(
    ids.into_iter().map(|ids| {
        let path = format!("https://inventory.roblox.com/v1/users/{}/items/Asset/{}", uid, ids); //Create URL
        let send_fut = client.get(&path).send(); //Send HTTP request
        let cloned_found = found.clone(); //Clone of found needed to store info from within the async move block
        async move {
            match send_fut.await {
                Ok(resp) => {
                    match resp.text().await {
                        Ok(text) => {
                            let api: OwnershipAPI = serde_json::from_str(&text).unwrap(); //Create instance of OwnershipAPI from JSON
                            if api.data.len() == 0 { //Check if the item is NOT owned
                            } else {
                                cloned_found.lock().unwrap().push(api); //If owned, push OwnershipAPI instance into vector
                            }
                            
                            }
                        
                        Err(_) => println!("ERROR from {}", path),
                    }
                }
                Err(_) => println!("ERROR from {}", path),
            }
        }
    })
    ).buffer_unordered(50).collect::<Vec<()>>(); //50 concurrent requests, this number can be raised at risk of HTTP request failure
    fetches.await;

    let item_vector = &*found.lock().unwrap(); //Gets the vector containing owned items out of the ARC/MUTEX

    let mut item_str = String::from(""); //Empty string used to print the list of items in one go
    let mut item_count = 0; //Total item count
    let mut total_value = 0; //Total value
    let mut total_rap = 0; //Total RAP

    for api in item_vector { //Iterate through OwnershipAPI objects
        for item in &api.data { //Iterate through item instance objects
            let mut value = roli_items[&item.id].value; //Mutable variable for value
            let rap = roli_items[&item.id].rap; //Item RAP
            if value == -1 {
                value = rap //If value is -1, set value to RAP (Just like on the Rolimon's website)
            }
            total_value = total_value + value; //Add item value to total
            total_rap = total_rap + rap; //Add item RAP to total
            item_count = item_count + 1; //Add item count to total
            let addstr = format!("\n{} ||| {}", item.id, item.name); //Create string to add to item_str
            item_str.push_str(&addstr) //Adds addstr to item_str
        }
    }

    println!("{} item(s) found.", item_count); //Prints amount of items
    print!("{}", item_str); //Prints the item list
    println!("\nTotal value is R${}\nTotal RAP is R${}", total_value, total_rap); //Prints the total value and total RAP
    pause(); //Waits for enter keypress to end program (Useful if launched from explorer instead of cmd)

    Ok(())
}