# Stream Tool — 视频实时推流工具

基于 **Tauri 2 + Vue 3** 的桌面应用：将本地视频、摄像头、桌面画面以实时速率推流，每路生成唯一 RTSP / HLS 地址，可在 **VLC** 等播放器或局域网内实时观看。

## 功能

### v0.2.0 新增
- **摄像头推流**：枚举 DirectShow 设备，可选麦克风
- **桌面采集推流**：全屏采集（gdigrab），可选麦克风
- 直播源自动 H.264+AAC 转码，独立 RTSP / HLS 地址

### 核心能力
- 多路并行推流，每路独立流 ID 与地址
- RTSP（`rtsp://局域网IP:8554/{id}`）— 推荐 VLC 播放，低延迟
- HLS（`http://局域网IP:8888/{id}/index.m3u8`）— 应用内预览
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
拖拽或选择视频文件 → 开始推流 → 复制 RTSP 地址到 VLC

### 摄像头 / 桌面推流
点击「摄像头推流」或「桌面采集推流」→ 选择设备 → 添加 → 开始推流

## VLC 播放

1. 开始推流
2. 复制 **RTSP** 地址（局域网 IP 版本）
3. VLC → 媒体 → 打开网络串流 → 粘贴地址

局域网访问需在防火墙放行 **8554**（RTSP）与 **8888**（HLS）。

## 架构

```
Vue UI → Tauri Rust → FFmpeg → MediaMTX → RTSP / HLS → VLC / 预览
         文件 -re / 摄像头 dshow / 桌面 gdigrab
```

## 许可证

MIT
