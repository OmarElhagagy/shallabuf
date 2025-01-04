#![no_main]

#[no_mangle]
pub extern "C" fn host_func(param: i32) -> i32 {
    param
}

#[no_mangle]
pub extern "C" fn _start() {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            eprintln!("Failed to build runtime: {err:?}");
            return;
        }
    };

    rt.block_on(async {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        let client = reqwest::Client::new();

        let body = serde_json::json!({
            "model": "dall-e-3",
            "prompt": "a white siamese cat",
            "n": 1,
            "size": "1024x1024"
        });

        let resp = client
            .post("https://api.openai.com/v1/images/generations")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await;

        match resp {
            Ok(r) => {
                if let Ok(json_resp) = r.text().await {
                    println!("OpenAI response: {json_resp}");
                }
            }
            Err(err) => eprintln!("Error sending request: {err:?}"),
        }
    });
}
