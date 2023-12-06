// mod cache;

// use cache::CacheKV;

use std::vec;
use worker::*;
use serde::{Serialize, Deserialize};
use regex::Regex;
use rand_distr::{Normal, Distribution};
use wasm_bindgen::prelude::*;


#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub id: u32,
    pub taken: bool,
    // reservation details, only filled out if taken=true
    pub res_email: Option<String>,
    pub res_name: Option<String>,
    pub res_card: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub key: String,
    pub id: String,
    pub version: u32,
    pub value: Ticket,
}

// type RWSet = Vec<String>;

// // create new tickets 
// // expected request body with number
// #[wasm_bindgen]
// pub async fn populate_tickets(n: u32) {
//     let cache = CacheKV::new().await;

//     // create n tickets 
//     for i in 0..n {
//         let key = format!("ticket-{i}");
//         let ticket = Ticket { 
//             id: i,
//             taken: false,
//             res_email: None,
//             res_name: None,
//             res_card: None,
//         };
//         let val = Item {
//             key: key.clone(),
//             id: key.clone(),
//             version: 0,
//             value: ticket,
//         };

//         cache.put(&key, &val).await.unwrap();
//     }
    
//     // save in cache so we can know how much to clear later
    
//     cache.put("count", &n).await.unwrap();
// }

// // clear the entire cache of tickets
// #[wasm_bindgen]
// pub async fn clear_cache() {
//     let cache = CacheKV::new().await;
//     // get count of how many tickets total
//     let n = cache.get::<u32>("count").await.unwrap().unwrap();

//     for i in 0..n {
//         cache.delete(&format!("ticket-{i}")).await.unwrap();
//     }

//     // reset count
//     cache.put("count", &0).await.unwrap();
// }

// // return a specific ticket
// #[wasm_bindgen]
// pub async fn get_ticket(ticket_id: u32) -> Option<JsValue> {
    
//     let cache = CacheKV::new().await;
//     match cache.get::<Item>(&format!("ticket-{ticket_id}")).await.unwrap() {
//         Some(val) => {
//             Some(serde_wasm_bindgen::to_value::<Ticket>(&val.value).unwrap())
//         },
//         None => {
//             None
//         }
//     }
// }

// multiply an input vector by a random normal matrix, returning an output vector
fn multiply_random_normal(input_vec: Vec<f32>, output_dim: usize, scale: f32) -> Vec<f32> {
    let normal = Normal::new(0.0, scale).unwrap();

    let mut normal_matrix = vec![vec![0f32; input_vec.len()]; output_dim];
    for i in 0..normal_matrix.len() {
        for j in 0..normal_matrix[i].len() {
            normal_matrix[i][j] = normal.sample(&mut rand::thread_rng());
        }
    }

    // output = (normal_matrix)(input_vec)
    let mut output = vec![0f32; output_dim];
    for i in 0..output.len() {
        for j in 0..input_vec.len() {
            output[i] += normal_matrix[i][j] * input_vec[j];
        }
    }

    output
}

// compute relu of input vector
fn relu(input_vec: Vec<f32>) -> Vec<f32> {
    let mut output = vec![0f32; input_vec.len()];
    for i in 0..output.len() {
        if input_vec[i] > 0.0 {
            output[i] = input_vec[i];
        }
    }

    output
}

// check if ticket reservation passes anti fraud test
// true means reservation is ok, false means not
#[wasm_bindgen]
pub fn anti_fraud(_ticket_id: u32, res_email: String, res_name: String, res_card: String) -> bool {
    // valid email must have some valid characters before @, some after, a dot, then some more
    const EMAIL_REGEX: &str = r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)";
    let re = Regex::new(EMAIL_REGEX).unwrap();

    if !re.is_match(&res_email.clone()) {
        return false;
    }

    // check "ml" model
    // create a feature vector from the name, email, and card
    let feature_str = [
        res_name.clone().as_bytes(), 
        res_email.clone().as_bytes(), 
        res_card.clone().as_bytes(),
    ].concat();
    // feature vector is normalized
    let mut feature_vec = vec![0f32; feature_str.len()];
    let mut feature_norm = 0.0;
    for i in 0..feature_str.len() {
        feature_norm += (feature_str[i] as f32).powi(2);
    }
    for i in 0..feature_vec.len() {
        feature_vec[i] = (feature_str[i] as f32) / (feature_norm.sqrt());
    }

    let model_depth = 256;
    for i in 0..model_depth {
        feature_vec = multiply_random_normal(feature_vec, 128, ((i % 64)+1) as f32);
        feature_vec = relu(feature_vec);
    }

    feature_norm = 0.0;
    for i in 0..feature_vec.len() {
        feature_norm += feature_vec[i].powi(2);
    }
    feature_norm > 0.0
}

// // reserve a ticket
// // expect request as a json with form of Ticket
// #[wasm_bindgen]
// pub async fn reserve_ticket(ticket_id: u32, res_email: String, res_name: String, res_card: String) -> bool {
//     let cache = CacheKV::new().await;

//     // get old val to compute new version number
//     let key = format!("ticket-{ticket_id}");
//     let resp = cache.get::<Item>(&key).await.unwrap();
//     if resp.is_none() {
//         return false;
//     }
    
//     let old_val = resp.unwrap();
//     let new_version = old_val.version + 1;
//     // check that the ticket is not already taken
//     if old_val.value.taken {
//         return false;
//     }
    
//     // create new ticket that is taken while checking reservation details are given
//     let new_ticket = Ticket {
//         id: ticket_id,
//         taken: true,
//         res_email: Some(res_email),
//         res_name: Some(res_name),
//         res_card: Some(res_card),
//     };

//     // call anti fraud detection
//     if !anti_fraud(&new_ticket) {
//         return false;
//     }

//     let new_val = Item {
//         key: key.clone(),
//         id: key.clone(),
//         version: new_version,
//         value: new_ticket,
//     };

//     // put back into cache
//     cache.put(&key, &new_val).await.unwrap();

//     true
// }

// extract the read write set from the request
#[wasm_bindgen]
pub async fn get_rw_set(ticket_id: u32, _res_email: String, _res_name: String, _res_card: String) -> String {
    format!("ticket-{ticket_id}")
}
