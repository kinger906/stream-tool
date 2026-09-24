import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, StreamTaskInfo, SystemInfo } from "../types";

export const api = {
  getSystemInfo: () => invoke<SystemInfo>("get_system_info"),
  listStreams: () => invoke<StreamTaskInfo[]>("list_streams"),
  addStreams: (paths: string[]) => invoke<StreamTaskInfo[]>("add_streams", { paths }),
  startStream: (id: string) => invoke<StreamTaskInfo>("start_stream", { id }),
  stopStream: (id: string) => invoke<StreamTaskInfo>("stop_stream", { id }),
  startAllStreams: () => invoke<StreamTaskInfo[]>("start_all_streams"),
  stopAllStreams: () => invoke<StreamTaskInfo[]>("stop_all_streams"),
  removeStream: (id: string) => invoke<void>("remove_stream", { id }),
  updateStream: (
    id: string,
    loopEnabled?: boolean,
    copyMode?: boolean,
  ) => invoke<StreamTaskInfo>("update_stream", { id, loopEnabled, copyMode }),
  updateSettings: (maxConcurrent: number) =>
    invoke<AppSettings>("update_settings", { maxConcurrent }),
  startMediamtx: () => invoke<void>("start_mediamtx"),
};
