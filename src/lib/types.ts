export interface ImageFile {
  id: string;
  name: string;
  file: string;
  thumbnail: string;
  width: number;
  height: number;
}
export interface Style {
  version: number;
  direction: string;
  palette: string;
  material: string;
  lighting: string;
  perspective: string;
  composition: string;
  negative: string;
  background: string;
  aspect: 'square' | 'portrait' | 'landscape';
  references: ImageFile[];
}
export type JobKind = 'generate' | 'variant' | 'upscale';
export type JobStatus = 'queued' | 'generating' | 'ready' | 'failed' | 'cancelled';
export interface GenerationRequest {
  schemaVersion: number;
  style: Style;
  subject: string;
  assetType: string;
  prompt: string;
  source: ImageFile | null;
}
export interface Generation {
  id: string;
  image: ImageFile;
  kind: JobKind;
  createdAt: number;
  request: GenerationRequest;
  provider: string;
  sourceResultId: string | null;
}
export interface Asset {
  id: string;
  name: string;
  subject: string;
  results: Generation[];
  preferredId: string | null;
  approved: boolean;
}
export interface Job {
  id: string;
  assetId: string;
  assetName: string;
  kind: JobKind;
  status: JobStatus;
  request: GenerationRequest;
  sourceResultId: string | null;
  attempts: number;
  createdAt: number;
  finishedAt: number | null;
  retryAt: number;
  error: string | null;
}
export interface Project {
  schemaVersion: number;
  id: string;
  name: string;
  description: string;
  assetType: string;
  createdAt: number;
  updatedAt: number;
  revision: number;
  style: Style;
  styleHistory: Style[];
  assets: Asset[];
  jobs: Job[];
  paused: boolean;
}
export interface ProjectSummary {
  id: string;
  name: string;
  assetType: string;
  count: number;
  updatedAt: number;
}
export interface Library {
  projects: ProjectSummary[];
  deletedProjects: ProjectSummary[];
  lastProjectId: string | null;
  root: string;
  warnings: string[];
}
export interface ProviderStatus {
  installed: boolean;
  authenticated: boolean;
  message: string;
  version: string | null;
}
export interface AssetInput {
  name: string;
  subject: string;
}
export type Action =
  | { type: 'addAssets'; items: AssetInput[] }
  | { type: 'editAsset'; assetId: string; name: string; subject: string }
  | { type: 'appendInstructions'; ids: string[]; instructions: string }
  | { type: 'deleteAssets' | 'duplicate' | 'reorder'; ids: string[] }
  | { type: 'approve'; ids: string[]; approved: boolean }
  | { type: 'prefer'; assetId: string; resultId: string }
  | { type: 'saveStyle'; style: Style; expectedVersion: number }
  | { type: 'useReference'; assetId: string; resultId: string }
  | { type: 'removeReference'; referenceId: string }
  | { type: 'enqueue'; ids: string[]; kind: JobKind }
  | { type: 'cancel' | 'retry'; ids: string[] }
  | { type: 'pause'; paused: boolean };
export type Mode = 'assets' | 'style' | 'queue' | 'export' | 'settings' | 'collections';
export type Filter = 'all' | 'draft' | 'ready' | 'approved' | 'failed';
export interface ExportOptions {
  ids: string[];
  version: 'preferred' | 'original' | 'upscaled';
  metadata: boolean;
  prefix: string;
}
export interface ExportReport {
  count: number;
  skipped: number;
  directory: string;
}
