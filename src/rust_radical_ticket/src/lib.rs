// mod cache;

// use cache::CacheKV;

use std::vec;
use rand::Rng;
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

// multiply an input vector by a random normal matrix, returning an output vector
fn multiply_random_normal(input_vec: Vec<f32>, output_dim: usize, scale: f32) -> Vec<f32> {
    let normal = Normal::new(0.0, scale).unwrap();

    let mut normal_matrix = vec![vec![0f32; input_vec.len()]; output_dim];
    for i in 0..normal_matrix.len() {
        for j in 0..normal_matrix[i].len() {
            // normal_matrix[i][j] = normal.sample(&mut rand::thread_rng());
            normal_matrix[i][j] = rand::thread_rng().gen_range(0.0..scale);
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
        feature_norm += 1.0;
    }
    feature_norm > 0.0
}
//
// extract the read write set from the request
#[wasm_bindgen]
pub async fn get_rw_set(ticket_id: u32, _res_email: String, _res_name: String, _res_card: String) -> String {
    format!("ticket-{ticket_id}")
}
