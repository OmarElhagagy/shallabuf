#![no_main]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenAIResponseData {
    url: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenAIResponse {
    data: Vec<OpenAIResponseData>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    prompt: String,
}

#[no_mangle]
/// # Safety
/// The caller must ensure that `input_ptr` is a valid pointer to a null-terminated string.
/// The caller is also responsible for managing the memory of the returned pointer.
///
/// # Panics
/// This function will panic if `CString::new` fails, which occurs if the input contains internal null bytes.
pub unsafe extern "C" fn run(input_ptr: *const c_char) -> *const c_char {
    if input_ptr.is_null() {
        return std::ptr::null();
    }

    let input = CStr::from_ptr(input_ptr);

    let Ok(json_str) = input.to_str() else {
        return std::ptr::null();
    };

    let Ok(payload) = serde_json::from_str::<Payload>(json_str) else {
        return std::ptr::null();
    };

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("Failed to build runtime: {err:?}");
            return std::ptr::null();
        }
    };

    rt.block_on(async {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "model": "dall-e-3",
            "prompt": payload.prompt,
            "response_format": "url",
            "n": 1,
            "size": "1024x1024"
        });

        let Ok(response) = client
            .post("https://api.openai.com/v1/images/generations")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
        else {
            return std::ptr::null();
        };

        let Ok(response_json) = response.json::<OpenAIResponse>().await else {
            return std::ptr::null();
        };

        let url = &response_json.data[0].url;

        match CString::new(serde_json::json!({ "image": url }).to_string()) {
            Ok(data) => data.into_raw(),
            Err(_) => std::ptr::null(),
        }
    })
}
