export type StreamStatus = "idle" | "running" | "reconnecting" | "stopped" | "error";
export type SourceType = "file" | "camera" | "display" | "window" | "region";
export type QualityPreset = "low" | "medium" | "high";
export type PreviewMode = "hls" | "webrtc";

export interface CaptureDevice {
  name: string;
}

export interface CaptureDevices {
  video: CaptureDevice[];
  audio: CaptureDevice[];
}

export interface CaptureRegion {
  x: number;
  y: number;
  width: number;
  height: number;
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
  autoReconnect: boolean;
  qualityPreset: QualityPreset;
  audioDevice: string | null;
  windowTitle: string | null;
  region: CaptureRegion | null;
  rtmpUrl: string | null;
  rtspUrl: string;
  hlsUrl: string;
  webrtcUrl: string;
  rtspUrlLocal: string;
  hlsUrlLocal: string;
  webrtcUrlLocal: string;
  publicHlsUrl: string | null;
  error: string | null;
  elapsedSecs: number;
  durationSecs: number | null;
  fileSizeBytes: number | null;
  bitrateKbps: number | null;
  fps: number | null;
  reconnectAttempts: number;
}

export interface AppSettings {
  maxConcurrent: number;
  selectedLanIp: string | null;
  rtspUsername: string | null;
  rtspPassword: string | null;
  recordDir: string | null;
  autoReconnectDefault: boolean;
  publicBaseUrl: string | null;
  minimizeToTray: boolean;
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
  autoReconnectDefault?: boolean;
  publicBaseUrl?: string;
  minimizeToTray?: boolean;
}

export interface StreamUpdate {
  loopEnabled?: boolean;
  copyMode?: boolean;
  streamName?: string;
  recordEnabled?: boolean;
  qualityPreset?: QualityPreset;
  autoReconnect?: boolean;
  rtmpUrl?: string;
  region?: CaptureRegion;
}

export interface SceneStreamSpec {
  streamName: string;
  filename: string;
  path: string;
  sourceType: SourceType;
  loopEnabled: boolean;
  copyMode: boolean;
  recordEnabled: boolean;
  autoReconnect: boolean;
  qualityPreset: QualityPreset;
  audioDevice: string | null;
  windowTitle: string | null;
  region: CaptureRegion | null;
  rtmpUrl: string | null;
}

export interface ScenePreset {
  id: string;
  name: string;
  createdAt: string;
  streams: SceneStreamSpec[];
}
