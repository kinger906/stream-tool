# Stream Tool — 视频实时推流工具

基于 **Tauri 2 + Vue 3** 的桌面应用：将本地视频、摄像头、桌面或指定窗口以实时速率推流，每路生成可自定义名称的 RTSP / HLS / WebRTC 地址，可在 **VLC** 等播放器或局域网内实时观看。

## 功能

### v0.3.0 新增
- **自定义流名称**：路径 slug（2–32 位字母数字/下划线/连字符），便于记忆与分享
- **录制**：按流启用 MediaMTX 录制，可配置录制目录
- **画质预设**：低 / 中 / 高（360p / 720p / 1080p + 对应码率）
- **窗口采集**：选择可见窗口标题进行 gdigrab 采集
- **码率 / FPS 监控**：推流时从 FFmpeg 统计实时显示
- **RTSP/HLS 认证**：可选用户名密码（MediaMTX readUser/readPass）
- **多网卡 IP 选择**：分享地址时可选局域网 IP
- **二维码分享**：生成 RTSP 地址二维码
- **WebRTC 低延迟预览**：应用内 HLS / WebRTC 切换预览

### v0.2.0
- **摄像头推流**：枚举 DirectShow 设备，可选麦克风
- **桌面采集推流**：全屏采集（gdigrab），可选麦克风

### 核心能力
- 多路并行推流，每路独立流名称与地址
- RTSP（`rtsp://IP:8554/{name}`）— 推荐 VLC 播放，低延迟
- HLS（`http://IP:8888/{name}/index.m3u8`）— 应用内预览
- WebRTC（`http://IP:8889/{name}/`）— 低延迟预览
- 文件推流：循环播放、Copy/转码模式、并发上限设置
- Copy 失败自动尝试 H.264+AAC 转码

## 环境要求

- Node.js 18+
- Rust（[rustup](https://rustup.rs/)）
- Windows 10/11（首期目标平台，采集功能依赖 DirectShow / gdigrab）

## 快速开始

```bash
npm install
npm run fetch-sidecars
npm run tauri dev
npm run tauri build
```

## Sidecar 手动放置

| 文件 | 说明 |
|------|------|
| `ffmpeg-x86_64-pc-windows-msvc.exe` | [FFmpeg Windows builds](https://www.gyan.dev/ffmpeg/builds/) |
| `mediamtx-x86_64-pc-windows-msvc.exe` | [MediaMTX Releases](https://github.com/bluenviron/mediamtx/releases) |

## 使用说明

### 文件推流
拖拽或选择视频文件 → 编辑流名称（可选）→ 开始推流 → 复制 RTSP 地址到 VLC

### 摄像头 / 桌面 / 窗口推流
点击对应按钮 → 选择设备或窗口、画质预设 → 添加 → 开始推流

### 录制
在任务行勾选「录制」后开始推流，文件保存至设置中的录制目录（默认应用数据目录下的 `recordings`）。

### 认证
在设置中填写 RTSP/HLS 用户名与密码并应用，MediaMTX 将自动重启并生效。VLC 需使用带凭据的 RTSP URL。

## VLC 播放

1. 开始推流
2. 复制 **RTSP** 地址（局域网 IP 版本），或扫描二维码
3. VLC → 媒体 → 打开网络串流 → 粘贴地址

局域网访问需在防火墙放行 **8554**（RTSP）、**8888**（HLS）与 **8889**（WebRTC）。

## 架构

```
Vue UI → Tauri Rust → FFmpeg → MediaMTX → RTSP / HLS / WebRTC → VLC / 预览
         文件 -re / 摄像头 dshow / 桌面·窗口 gdigrab
```

## 许可证

MIT
