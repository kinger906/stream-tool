export type StreamStatus = "idle" | "running" | "stopped" | "error";
export type SourceType = "file" | "camera" | "display";

export interface CaptureDevice {
  name: string;
}

export interface CaptureDevices {
  video: CaptureDevice[];
  audio: CaptureDevice[];
}

export interface StreamTaskInfo {
  id: string;
  filename: string;
  path: string;
  sourceType: SourceType;
  status: StreamStatus;
  loopEnabled: boolean;
  copyMode: boolean;
  audioDevice: string | null;
  rtspUrl: string;
  hlsUrl: string;
  rtspUrlLocal: string;
  hlsUrlLocal: string;
  error: string | null;
  elapsedSecs: number;
  durationSecs: number | null;
}

export interface AppSettings {
  maxConcurrent: number;
}

export interface DependencyStatus {
  ffmpegAvailable: boolean;
  mediamtxAvailable: boolean;
  ffmpegPathHint: string;
  mediamtxPathHint: string;
}

export interface SystemInfo {
  lanIp: string;
  mediamtxRunning: boolean;
  dependencies: DependencyStatus;
  settings: AppSettings;
}
