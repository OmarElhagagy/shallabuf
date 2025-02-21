#![no_main]

use reqwest::blocking::get;
use std::ffi::CString;
use std::os::raw::c_char;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoinDeskUSD {
    rate: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CoinDeskResponse {
    bpi: CoinDeskUSD,
}

#[no_mangle]
/// # Safety
/// The caller must ensure that `input_ptr` is a valid pointer to a null-terminated string.
/// The caller is also responsible for managing the memory of the returned pointer.
///
/// # Panics
/// This function will panic if `CString::new` fails, which occurs if the input contains internal null bytes.
pub unsafe extern "C" fn run(_input_ptr: *const c_char) -> *const c_char {
    // let rt = match tokio::runtime::Builder::new_current_thread()
    //     .enable_all()
    //     .build()
    // {
    //     Ok(rt) => rt,
    //     Err(err) => {
    //         eprintln!("Failed to build runtime: {err:?}");
    //         return std::ptr::null();
    //     }
    // };

    let response = get("https://api.github.com").unwrap();
    let response_json = response.json::<CoinDeskResponse>().unwrap();

    match CString::new(serde_json::json!({ "price": &response_json.bpi.rate }).to_string()) {
        Ok(data) => data.into_raw(),
        Err(_) => std::ptr::null(),
    }

    // rt.block_on(async {
    //     let client = match reqwest::Client::builder().build() {
    //         Ok(client) => client,
    //         Err(err) => {
    //             eprintln!("Failed to create HTTP client: {err:?}");
    //             return std::ptr::null();
    //         }
    //     };

    //     let Ok(response) = client
    //         .get("https://api.coindesk.com/v1/bpi/currentprice.json")
    //         .send()
    //         .await
    //     else {
    //         return std::ptr::null();
    //     };

    //     let Ok(response_json) = response.json::<CoinDeskResponse>().await else {
    //         return std::ptr::null();
    //     };

    //     match CString::new(serde_json::json!({ "price": &response_json.bpi.rate }).to_string()) {
    //         Ok(data) => data.into_raw(),
    //         Err(_) => std::ptr::null(),
    //     }
    // })
}
