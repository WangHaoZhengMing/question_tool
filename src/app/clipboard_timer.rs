use super::AppState;
use crate::App;
use slint::{Image, Timer, Weak};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 剪贴板定时器管理
pub struct ClipboardTimer {
    app_state: Arc<AppState>,
    timer: Option<Timer>,
}

impl ClipboardTimer {
    /// 创建新的剪贴板定时器
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { 
            app_state,
            timer: None,
        }
    }

    /// 启动定时器监控剪贴板变化
    pub fn start(&mut self, app_weak: Weak<App>) {
        tracing::info!("[clipboard_timer] Starting UI timer for clipboard polling");

        let path_monitor = self.app_state.clipboard_path.clone();
        let current_image_path = self.app_state.current_image_path.clone();

        let timer = Timer::default();
        tracing::info!("[clipboard_timer] Starting clipboard check(before function)");
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(1000),
            move || {
                Self::handle_clipboard_check(&path_monitor, &current_image_path, &app_weak);
            },
        );
        
        // 保存定时器以保持其生命周期
        self.timer = Some(timer);
    }

    /// 处理剪贴板检查逻辑
    fn handle_clipboard_check(
        path_monitor: &Arc<Mutex<Option<PathBuf>>>,
        current_image_path: &Arc<Mutex<Option<PathBuf>>>,
        app_weak: &Weak<App>,
    ) {
        if let Ok(path_lock) = path_monitor.lock() {
            if let Some(ref path) = *path_lock {
                tracing::debug!(
                    "[clipboard_timer] New clipboard image detected: {}",
                    path.display()
                );

                if let Some(app) = app_weak.upgrade() {
                    match Image::load_from_path(path) {
                        Ok(image) => {
                            tracing::info!(
                                "[clipboard_timer] Successfully loaded and displayed image"
                            );
                            
                            // 释放旧图片的内存引用
                            app.set_current_image(slint::Image::default());
                            
                            // 设置新图片
                            app.set_current_image(image);

                            // 更新当前图片路径
                            if let Ok(mut img_path_lock) = current_image_path.lock() {
                                *img_path_lock = Some(path.clone());
                            }

                            // 清除路径以避免重复处理
                            drop(path_lock);
                            if let Ok(mut path_lock) = path_monitor.lock() {
                                *path_lock = None;
                            }
                        }
                        Err(e) => {
                            tracing::info!(
                                "[clipboard_timer] Failed to load image: {} - Error: {:?}",
                                path.display(),
                                e
                            );
                        }
                    }
                } else {
                    tracing::info!("[clipboard_timer] App upgrade failed");
                }
            }
        }
    }
}
