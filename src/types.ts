export type StreamStatus = "idle" | "running" | "stopped" | "error";
export type SourceType = "file" | "camera" | "display" | "window";
export type QualityPreset = "low" | "medium" | "high";
export type PreviewMode = "hls" | "webrtc";

export interface CaptureDevice {
  name: string;
}

export interface CaptureDevices {
  video: CaptureDevice[];
  audio: CaptureDevice[];
}

export interface StreamTaskInfo {
  id: string;
  streamName: string;
  filename: string;
  path: string;
  sourceType: SourceType;
  status: StreamStatus;
  loopEnabled: boolean;
  copyMode: boolean;
  recordEnabled: boolean;
  qualityPreset: QualityPreset;
  audioDevice: string | null;
  windowTitle: string | null;
  rtspUrl: string;
  hlsUrl: string;
  webrtcUrl: string;
  rtspUrlLocal: string;
  hlsUrlLocal: string;
  webrtcUrlLocal: string;
  error: string | null;
  elapsedSecs: number;
  durationSecs: number | null;
  bitrateKbps: number | null;
  fps: number | null;
}

export interface AppSettings {
  maxConcurrent: number;
  selectedLanIp: string | null;
  rtspUsername: string | null;
  rtspPassword: string | null;
  recordDir: string | null;
}

export interface DependencyStatus {
  ffmpegAvailable: boolean;
  mediamtxAvailable: boolean;
  ffmpegPathHint: string;
  mediamtxPathHint: string;
}

export interface SystemInfo {
  lanIp: string;
  lanIps: string[];
  mediamtxRunning: boolean;
  dependencies: DependencyStatus;
  settings: AppSettings;
  recordDir: string;
}

export interface SettingsUpdate {
  maxConcurrent?: number;
  selectedLanIp?: string;
  rtspUsername?: string;
  rtspPassword?: string;
  recordDir?: string;
}

export interface StreamUpdate {
  loopEnabled?: boolean;
  copyMode?: boolean;
  streamName?: string;
  recordEnabled?: boolean;
  qualityPreset?: QualityPreset;
}
