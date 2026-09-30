import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  CaptureDevice,
  CaptureDevices,
  CaptureRegion,
  ScenePreset,
  SettingsUpdate,
  StreamTaskInfo,
  StreamUpdate,
  SystemInfo,
} from "../types";

export const api = {
  getSystemInfo: () => invoke<SystemInfo>("get_system_info"),
  listLanIps: () => invoke<string[]>("list_lan_ips"),
  listStreams: () => invoke<StreamTaskInfo[]>("list_streams"),
  addStreams: (paths: string[]) => invoke<StreamTaskInfo[]>("add_streams", { paths }),
  listCaptureDevices: () => invoke<CaptureDevices>("list_capture_devices"),
  listWindows: () => invoke<CaptureDevice[]>("list_windows"),
  addCameraStream: (videoDevice: string, audioDevice?: string) =>
    invoke<StreamTaskInfo>("add_camera_stream", {
      videoDevice,
      audioDevice: audioDevice ?? null,
    }),
  addDisplayStream: (audioDevice?: string) =>
    invoke<StreamTaskInfo>("add_display_stream", {
      audioDevice: audioDevice ?? null,
    }),
  addWindowStream: (windowTitle: string, audioDevice?: string) =>
    invoke<StreamTaskInfo>("add_window_stream", {
      windowTitle,
      audioDevice: audioDevice ?? null,
    }),
  addRegionStream: (region: CaptureRegion, audioDevice?: string) =>
    invoke<StreamTaskInfo>("add_region_stream", {
      region,
      audioDevice: audioDevice ?? null,
    }),
  startStream: (id: string) => invoke<StreamTaskInfo>("start_stream", { id }),
  stopStream: (id: string) => invoke<StreamTaskInfo>("stop_stream", { id }),
  startAllStreams: () => invoke<StreamTaskInfo[]>("start_all_streams"),
  stopAllStreams: () => invoke<StreamTaskInfo[]>("stop_all_streams"),
  removeStream: (id: string) => invoke<void>("remove_stream", { id }),
  updateStream: (id: string, update: StreamUpdate) =>
    invoke<StreamTaskInfo>("update_stream", { id, ...update }),
  updateSettings: (update: SettingsUpdate) =>
    invoke<AppSettings>("update_settings", { ...update }),
  listScenes: () => invoke<ScenePreset[]>("list_scenes"),
  saveScene: (name: string) => invoke<ScenePreset>("save_scene", { name }),
  deleteScene: (id: string) => invoke<void>("delete_scene", { id }),
  applyScene: (id: string, replace = true) =>
    invoke<StreamTaskInfo[]>("apply_scene", { id, replace }),
  startMediamtx: () => invoke<void>("start_mediamtx"),
};
