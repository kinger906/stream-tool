# Stream Tool — 视频实时推流工具

基于 **Tauri 2 + Vue 3** 的桌面应用：将本地视频文件以实时速率推流，每路生成唯一 RTSP / HLS 地址，可在 **VLC** 等播放器或局域网内实时观看。

## 功能

- 多视频并行推流，每路独立流 ID 与地址
- RTSP（`rtsp://局域网IP:8554/{id}`）— 推荐 VLC 播放，低延迟
- HLS（`http://局域网IP:8888/{id}/index.m3u8`）— 应用内预览
- 循环播放、Copy/转码模式、并发上限设置
- Copy 失败自动尝试 H.264+AAC 转码
- 摄像头 / 桌面采集（UI 预留，后续版本）

## 环境要求

- Node.js 18+
- Rust（[rustup](https://rustup.rs/)）
- Windows 10/11（首期目标平台）

## 快速开始

```bash
# 1. 安装依赖
npm install

# 2. 下载 FFmpeg + MediaMTX sidecar（约 100MB+）
npm run fetch-sidecars

# 3. 开发运行
npm run tauri dev

# 4. 打包
npm run tauri build
```

## Sidecar 手动放置

若自动下载失败，请将二进制放入 `src-tauri/binaries/`：

| 文件 | 说明 |
|------|------|
| `ffmpeg-x86_64-pc-windows-msvc.exe` | [FFmpeg Windows builds](https://www.gyan.dev/ffmpeg/builds/) |
| `mediamtx-x86_64-pc-windows-msvc.exe` | [MediaMTX Releases](https://github.com/bluenviron/mediamtx/releases) |

## VLC 播放

1. 添加视频并开始推流
2. 复制 **RTSP** 地址（局域网 IP 版本）
3. VLC → 媒体 → 打开网络串流 → 粘贴地址

局域网访问需在防火墙放行 **8554**（RTSP）与 **8888**（HLS）。

## 架构

```
Vue UI → Tauri Rust → FFmpeg (-re 推流) → MediaMTX → RTSP / HLS
```

## 许可证

MIT
