use arboard::Clipboard;
use image::ImageEncoder;
use std::path::PathBuf;
use std::fs::File;
use std::sync::{Arc, Mutex};

pub fn start_clipboard_monitor() -> Arc<Mutex<Option<PathBuf>>> {
    tracing::info!("[clipboard_monitor] Monitor thread starting");
    
    let current_path_handle = std::sync::Arc::new(std::sync::Mutex::new(None));
    let handle_clone = current_path_handle.clone();
    let last_saved_file: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    
    std::thread::spawn(move || {
        tracing::debug!("[clipboard_monitor] Thread spawned");
        let mut last_clipboard_hash = 0u64;
        let mut check_count = 0u32;
        let last_file_clone = last_saved_file.clone();
        
        loop {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            check_count += 1;
            // 只在前几次检查时打印提示信息
            if check_count <= 5 {
                tracing::debug!("[clipboard_monitor] Checking clipboard... (check #{})", check_count);
            } else if check_count % 30 == 0 {
                // 每60秒打印一次心跳信息
                tracing::trace!("[clipboard_monitor] Still monitoring... (check #{})", check_count);
            }
            
            // 创建剪贴板实例
            let mut clipboard = match Clipboard::new() {
                Ok(clipboard) => clipboard,
                Err(e) => {
                    tracing::error!("[clipboard_monitor] Failed to create clipboard: {:?}", e);
                    continue;
                }
            };
            
            // 前几次检测时显示文本访问状态
            if check_count <= 3 {
                match clipboard.get_text() {
                    Ok(text) => tracing::debug!("[clipboard_monitor] Clipboard text access OK, text length: {}", text.len()),
                    Err(e) => tracing::trace!("[clipboard_monitor] Clipboard text access failed: {:?}", e),
                }
            }
            
            // 尝试获取图片
            let image = match clipboard.get_image() {
                Ok(image) => {
                    tracing::trace!("[clipboard_monitor] Found image in clipboard");
                    image
                },
                Err(e) => {
                    let error_msg = format!("{:?}", e);
                    if !error_msg.contains("ContentNotAvailable") && !error_msg.contains("GetClipboardData") && check_count <= 5 {
                        tracing::error!("[clipboard_monitor] Clipboard error: {:?}", e);
                    }
                    if last_clipboard_hash != 0 {
                        tracing::debug!("[clipboard_monitor] No image in clipboard anymore");
                        last_clipboard_hash = 0;
                    }
                    continue;
                }
            };
            
            // 计算图片哈希值
            let mut image_hash = (image.bytes.len() as u64)
                .wrapping_mul(image.width as u64)
                .wrapping_mul(image.height as u64);
            
            if image.bytes.len() >= 16 {
                for i in 0..16 {
                    image_hash = image_hash.wrapping_add(image.bytes[i] as u64 * (i as u64 + 1));
                }
            }
            
        tracing::debug!("[clipboard_monitor] Found image in clipboard: {}x{}, {} bytes, hash: {}, last_hash: {}", 
            image.width, image.height, image.bytes.len(), image_hash, last_clipboard_hash);
            
            // 检查是否是新图片
            if image_hash != last_clipboard_hash {
                tracing::info!("[clipboard_monitor] New image detected!");
                last_clipboard_hash = image_hash;
                
                // 删除旧的临时文件以释放磁盘空间和内存
                if let Ok(mut last_file) = last_file_clone.lock() {
                    if let Some(old_path) = last_file.take() {
                        if old_path.exists() {
                            match std::fs::remove_file(&old_path) {
                                Ok(_) => tracing::debug!("[clipboard_monitor] Deleted old temp file: {}", old_path.display()),
                                Err(e) => tracing::warn!("[clipboard_monitor] Failed to delete old temp file: {}", e),
                            }
                        }
                    }
                }
                
                // 保存图片
                let temp_dir = std::env::temp_dir();
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let file_path = temp_dir.join(format!("slint_paste_{}.png", timestamp));
                tracing::debug!("[clipboard_monitor] Saving image to: {}", file_path.to_string_lossy());
                
                // 保存新文件路径到 last_saved_file
                if let Ok(mut last_file) = last_file_clone.lock() {
                    *last_file = Some(file_path.clone());
                }
                
                // 更新共享路径句柄
                if let Ok(mut handle_path) = handle_clone.lock() {
                    *handle_path = Some(file_path.clone());
                    tracing::info!("[clipboard_monitor] Updated shared path handle");
                }else{
                    tracing::info!("[clipboard_monitor] Failed to lock shared path handle");
                }
                // 保存图片文件
                if let Ok(mut file) = File::create(&file_path) {
                    let buffer = &image.bytes;
                    let (width, height) = (image.width as u32, image.height as u32);
                    let encoder = image::codecs::png::PngEncoder::new(&mut file);
                    let bytes_per_pixel = buffer.len() / (width * height) as usize;
                    let color_type = match bytes_per_pixel {
                        4 => image::ColorType::Rgba8,
                        3 => image::ColorType::Rgb8,
                        1 => image::ColorType::L8,
                        _ => image::ColorType::Rgba8,
                    };
                    tracing::debug!("[clipboard_monitor] Image details: {}x{}, {} bytes, {} bytes/pixel", width, height, buffer.len(), bytes_per_pixel);
                    if encoder.write_image(buffer, width, height, color_type.into()).is_ok() {
                        tracing::info!("[clipboard_monitor] Image saved successfully");
                    } else if color_type != image::ColorType::Rgba8 {
                        if let Ok(mut file) = File::create(&file_path) {
                            let encoder = image::codecs::png::PngEncoder::new(&mut file);
                            if encoder.write_image(buffer, width, height, image::ColorType::Rgba8.into()).is_ok() {
                                tracing::info!("[clipboard_monitor] Image saved with fallback RGBA8 format");
                            } else {
                                tracing::error!("[clipboard_monitor] Failed to save image (fallback RGBA8)");
                            }
                        }
                    } else {
                        tracing::error!("[clipboard_monitor] Failed to save image");
                    }
                } else {
                    tracing::error!("[clipboard_monitor] Failed to create file for image");
                }
            }
        }
    });
    
    current_path_handle
}