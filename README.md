# Stream Tool — 视频实时推流工具

基于 **Tauri 2 + Vue 3** 的桌面应用：将本地视频、摄像头、桌面、窗口或屏幕区域以实时速率推流，每路生成可自定义名称的 RTSP / HLS / WebRTC 地址，并可同时 RTMP 转推到直播平台。

## 功能

### v0.4.0 新增
- **RTMP 转推**：每路可配置 `rtmp://` / `rtmps://` 地址，与本地 RTSP 同步推送
- **自动重连**：推流意外中断后按退避策略自动恢复（可单路开关）
- **区域截取**：指定屏幕区域 (x/y/宽/高) 采集推流
- **场景预设**：保存当前任务列表，一键加载替换
- **公网分享**：填写 Cloudflare Tunnel 等 Base URL，生成公网 HLS 链接与二维码
- **系统托盘**：关闭窗口可最小化到托盘，托盘菜单支持全部开始/停止

### v0.3.x
- 自定义流名称、录制、画质预设、窗口采集、码率监控、认证、多网卡 IP、WebRTC 预览
- 修复 FFmpeg 7 dshow 设备名引号导致的 I/O error、窗口标题中文乱码

### 核心能力
- 多路并行推流，每路独立流名称与地址
- RTSP / HLS / WebRTC 本地分发；可选 RTMP 外推
- 文件：循环、Copy/转码；采集源：摄像头 / 桌面 / 窗口 / 区域

## 环境要求

- Node.js 18+
- Rust（[rustup](https://rustup.rs/)）
- Windows 10/11

## 快速开始

```bash
npm install
npm run fetch-sidecars
npm run tauri dev
npm run tauri build
```

## 使用提示

1. 添加源 → 可选设置 RTMP 地址、画质、录制、自动重连 → **开始**
2. 确认状态为「推流中」后再预览或用 VLC 打开 RTSP
3. 公网分享：`cloudflared tunnel --url http://127.0.0.1:8888`，把 HTTPS 地址填入设置
4. 关闭窗口默认进入托盘；托盘右键可退出

防火墙放行 **8554**（RTSP）、**8888**（HLS）、**8889**（WebRTC）。

## 架构

```
Vue UI → Tauri Rust → FFmpeg → MediaMTX → RTSP / HLS / WebRTC
                              ↘ (可选) RTMP 直播平台
```

## 许可证

MIT
