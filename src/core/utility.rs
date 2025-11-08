use base64::{Engine, engine::general_purpose};
use image::ImageFormat;
use std::path::Path;

pub fn img_to_base64_withpath(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // 打开并解码图片，然后编码为 PNG 格式字节流，最后转为 base64 字符串
    let image = image::ImageReader::open(path)?.decode()?;
    let mut buf = Vec::new();
    image.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)?;
    Ok(general_purpose::STANDARD.encode(&buf))
}


pub fn decode_binary_to_str(key: &str) -> String {
    let bytes: Vec<u8> = key
            .split(',')
            .filter_map(|b| u8::from_str_radix(b.trim(), 2).ok())
            .collect();
    String::from_utf8(bytes).unwrap()
}