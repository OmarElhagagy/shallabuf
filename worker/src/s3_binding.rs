use std::ffi::CString;
use std::os::raw::c_char;

pub async fn upload_file(data: Vec<u8>) -> Option<String> {
    let minio_endpoint = std::env::var("MINIO_ENDPOINT").ok()?;
    let minio_access_key = std::env::var("MINIO_ACCESS_KEY").ok()?;
    let minio_secret_key = std::env::var("MINIO_SECRET_KEY").ok()?;
    let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "generated-images".to_string());

    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(minio_endpoint.clone())
        .force_path_style(true)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            minio_access_key,
            minio_secret_key,
            None,
            None,
            "",
        ))
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let key = format!("generated_image_{}.png", now);

    if s3_client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .body(aws_sdk_s3::primitives::ByteStream::from(data))
        .send()
        .await
        .is_err()
    {
        return None;
    }

    Some(format!("{}/{}", minio_endpoint, key))
}

// Host binding to be registered with WASM.
#[no_mangle]
pub extern "C" fn upload_file_binding(data_ptr: *const u8, len: usize) -> *const c_char {
    // Safety: The caller must provide a valid pointer to `len` bytes.
    let data = unsafe { std::slice::from_raw_parts(data_ptr, len) };
    let data_vec = data.to_vec();

    // Blocking on async upload; in production consider spawning an async task.
    let url = match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(upload_file(data_vec))
    {
        Some(url) => url,
        None => return std::ptr::null(),
    };

    match CString::new(url) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null(),
    }
}
