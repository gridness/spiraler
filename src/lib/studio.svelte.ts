import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type {
  Action,
  ExportOptions,
  ExportReport,
  ImageFile,
  Library,
  Project,
  ProviderStatus,
} from './types';

export const studio = $state({
  project: null as Project | null,
  library: null as Library | null,
  provider: null as ProviderStatus | null,
  loading: true,
  pending: 0,
  unsaved: false,
  error: '',
  notice: '',
  signingIn: false,
});
function accept(project: Project) {
  if (studio.project?.id === project.id && project.revision >= studio.project.revision)
    studio.project = project;
}
export function notify(message: string) {
  studio.notice = message;
}
export async function attempt<T>(work: () => Promise<T>): Promise<T | undefined> {
  studio.pending++;
  try {
    return await work();
  } catch (error) {
    studio.error = String(error);
    return undefined;
  } finally {
    studio.pending--;
  }
}
export async function refreshLibrary() {
  studio.library = await invoke<Library>('library');
}
export async function openProject(id: string) {
  await attempt(async () => {
    studio.project = await invoke<Project>('open_project', { projectId: id });
    await getCurrentWindow().setTitle(`${studio.project.name} · Spiraler`);
  });
}
export async function deleteProject(id: string) {
  await mutations;
  return attempt(async () => {
    await invoke('delete_project', { projectId: id });
    if (studio.project?.id === id) {
      studio.project = null;
      await getCurrentWindow().setTitle('Spiraler');
    }
    await refreshLibrary();
    notify('Collection deleted. Restore it from Deleted collections.');
    return true;
  });
}
export async function restoreProject(id: string) {
  return attempt(async () => {
    await invoke('restore_project', { projectId: id });
    await refreshLibrary();
    notify('Collection restored. Its queue is paused.');
  });
}
export async function initialize() {
  try {
    const unlisten = await listen<Project>('project-changed', ({ payload }) => {
      const previous = studio.project;
      if (previous?.id === payload.id) {
        const completed = payload.jobs.filter(
          (j) =>
            j.status === 'ready' &&
            previous.jobs.some((p) => p.id === j.id && p.status !== 'ready'),
        );
        if (completed.length) notify(`${completed[0].assetName} is ready to review.`);
        accept(payload);
      }
    });
    const unerror = await listen<string>('studio-error', ({ payload }) => {
      studio.error = payload;
    });
    await refreshLibrary();
    const last = studio.library?.lastProjectId;
    if (last && studio.library?.projects.some((p) => p.id === last)) await openProject(last);
    void attempt(async () => {
      studio.provider = await invoke<ProviderStatus>('provider_status');
    });
    return () => {
      unlisten();
      unerror();
    };
  } catch (error) {
    studio.error = `Could not open the studio. ${String(error)}`;
    return () => {};
  } finally {
    studio.loading = false;
  }
}
export async function createProject(name: string, description: string, assetType: string) {
  return attempt(async () => {
    studio.project = await invoke<Project>('create_project', { name, description, assetType });
    await refreshLibrary();
    return studio.project;
  });
}
let mutations = Promise.resolve();
export async function act(action: Action) {
  const projectId = studio.project?.id;
  if (!projectId) return;
  const operation = mutations.then(() =>
    attempt(async () => {
      const project = await invoke<Project>('mutate', { projectId, action });
      accept(project);
      return project;
    }),
  );
  mutations = operation.then(() => {});
  return operation;
}
export async function importImages(assetId: string | null = null, paths: string[] | null = null) {
  const projectId = studio.project?.id;
  if (!projectId) return;
  return attempt(async () => {
    const p = await invoke<Project | null>('import_images', { projectId, assetId, paths });
    if (p) {
      accept(p);
      notify(assetId ? 'Images added to this asset.' : 'Style references added.');
    }
  });
}
export async function importAssetList() {
  return attempt(() => invoke<string | null>('import_asset_list'));
}
export async function exportAssets(options: ExportOptions) {
  return attempt(async () => {
    const report = await invoke<ExportReport | null>('export_assets', {
      projectId: studio.project?.id,
      options,
    });
    if (report)
      notify(
        `Exported ${report.count} ${report.count === 1 ? 'image' : 'images'}${report.skipped ? `; ${report.skipped} had no matching result` : ''}.`,
      );
    return report;
  });
}
export async function reveal(imageId: string | null = null) {
  await attempt(() => invoke('reveal', { projectId: studio.project?.id, imageId }));
}
export async function connect() {
  studio.signingIn = true;
  await attempt(async () => {
    studio.provider = await invoke<ProviderStatus>('connect_provider');
  });
  studio.signingIn = false;
}
export async function checkConnection() {
  await attempt(async () => {
    studio.provider = await invoke<ProviderStatus>('provider_status');
  });
}
export async function installationHelp() {
  await attempt(() => invoke('open_help'));
}
export function imageUrl(image: ImageFile, thumbnail = true) {
  return convertFileSrc(
    `${studio.library?.root}/projects/${studio.project?.id}/${thumbnail ? image.thumbnail : image.file}`,
  );
}
