# 内存优化报告

## 问题诊断

你的程序内存占用不断增长（达到 349MB）的主要原因是**内存泄漏**，具体问题包括：

### 1. 临时图片文件累积 🔴 严重
- **问题**: 剪贴板监控每2秒检查一次，每次发现新图片就创建临时文件（`slint_paste_*.png`）
- **后果**: 临时文件永远不删除，占用磁盘和内存
- **影响**: 使用时间越长，文件越多，内存占用越大

### 2. 图片对象未释放 🟡 中等
- **问题**: 加载新图片时，旧图片的 Slint Image 对象仍在内存中
- **后果**: 每张图片的像素数据（可能几MB）持续占用内存

### 3. Tokio Runtime 重复创建 🟡 中等
- **问题**: 每次测试 LLM 连接都创建新的 tokio runtime
- **后果**: Runtime 未正确清理，线程池累积

## 修复方案

### ✅ 修复1: 自动删除旧临时文件
**文件**: `src/core/clipboard_monitor.rs`

```rust
// 新增：跟踪上次保存的文件
let last_saved_file: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));

// 检测到新图片时，先删除旧文件
if let Ok(mut last_file) = last_file_clone.lock() {
    if let Some(old_path) = last_file.take() {
        if old_path.exists() {
            std::fs::remove_file(&old_path).ok(); // 删除旧文件
        }
    }
}
```

**效果**: 只保留最新的一张临时图片，旧的自动删除

### ✅ 修复2: 释放旧图片内存
**文件**: `src/app/clipboard_timer.rs`

```rust
// 加载新图片前，先清空旧图片
app.set_current_image(slint::Image::default()); // 释放旧图片
app.set_current_image(image); // 设置新图片
```

**效果**: 确保旧图片的内存被正确释放

### ✅ 修复3: 使用全局 Tokio Runtime
**文件**: `src/app/app_state.rs`

```rust
use once_cell::sync::Lazy;

// 全局共享的 runtime，只创建一次
static TOKIO_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
});

// 使用全局 runtime
TOKIO_RUNTIME.block_on(async { ... })
```

**效果**: 避免 runtime 重复创建和线程泄漏

## 依赖优化

### 已优化的依赖配置:

```toml
[dependencies]
image = { version = "0.25.8", default-features = false, features = ["png"] }
# 只保留 PNG 支持，移除其他格式

tokio = { version = "1.47.1", features = ["sync", "rt", "macros"] }
# 精简 features，只保留必需的

tracing-subscriber = { version = "0.3.20", features = ["env-filter"] }
# 移除 chrono feature，减少依赖
```

### 移除的依赖:
- ❌ `syntect` - 代码中未使用

## 预期效果

优化后的内存表现：

- **启动内存**: ~50-80MB（取决于初始状态）
- **运行时内存**: 应保持稳定，不再持续增长
- **临时文件**: 只保留1个最新文件，而非累积
- **峰值内存**: 处理大图片时可能短暂升高，但会快速释放

## 使用建议

1. **重新编译**: 
   ```powershell
   cargo build --release
   ```

2. **清理旧临时文件**: 
   ```powershell
   Remove-Item "$env:TEMP\slint_paste_*.png"
   ```

3. **监控内存**: 
   - 使用任务管理器观察程序内存占用
   - 应该保持稳定，不再持续增长

4. **如果仍有问题**:
   - 检查是否有大量图片在 UI 历史中累积
   - 考虑添加图片历史限制（如最多保留10张）
   - 检查日志文件大小（tracing 可能累积）

## 进一步优化建议

如果内存仍然较高，可以考虑：

1. **限制图片分辨率**: 自动缩放大图片
2. **添加内存上限**: 监控并主动清理
3. **使用更轻量的日志**: 减少 tracing 的内存占用
4. **图片压缩**: 存储压缩后的图片而非原始像素

## 测试验证

编译后，运行程序并：
1. 多次复制不同图片到剪贴板
2. 观察内存是否稳定
3. 检查临时目录只有1个 `slint_paste_*.png` 文件

预期：内存应保持在 **100-150MB** 左右，不再持续增长。
