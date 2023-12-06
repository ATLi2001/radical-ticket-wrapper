use wasm_bindgen::prelude::*;
use regex::Regex;
use rand_distr::{Normal, Distribution};

#[wasm_bindgen]
extern "C" {
		#[wasm_bindgen(js_namespace = console)]
		fn log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

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
pub fn anti_fraud(res_email: Option<String>, res_name: Option<String>, res_card: Option<String>) -> bool {
    // valid email must have some valid characters before @, some after, a dot, then some more
    const EMAIL_REGEX: &str = r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)";
    let re = Regex::new(EMAIL_REGEX).unwrap();

		console_log!("Top of rust anti fraud call");
    if !re.is_match(&res_email.clone().unwrap()) {
				console_log!("No match on email regex");
        return false;
    }

    // check "ml" model
    // create a feature vector from the name, email, and card
    let feature_str = [
        res_name.clone().unwrap().as_bytes(),
        res_email.clone().unwrap().as_bytes(),
        res_card.clone().unwrap().as_bytes(),
    ].concat();
		console_log!("Created feature_str in rust");
    // feature vector is normalized
    let mut feature_vec = vec![0f32; feature_str.len()];
    let mut feature_norm = 0.0;
    for i in 0..feature_str.len() {
        feature_norm += (feature_str[i] as f32).powi(2);
    }
    for i in 0..feature_vec.len() {
        feature_vec[i] = (feature_str[i] as f32) / (feature_norm.sqrt());
    }

		console_log!("Doing the chunky stuff in rust");
    let model_depth = 256;
    for i in 0..model_depth {
        feature_vec = multiply_random_normal(feature_vec, 128, ((i % 64)+1) as f32);
        feature_vec = relu(feature_vec);
    }

    true
}

#[wasm_bindgen]
pub fn reserve_ticket() {
}

#[wasm_bindgen]
pub fn greet() {
	console_log!("Hello {}!", "world");
}
