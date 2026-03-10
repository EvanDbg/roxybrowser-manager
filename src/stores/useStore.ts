import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type {
    SyncOperationResult,
    SyncStatus,
    WebDavSnapshotMeta,
} from "../types/settings";

export interface UserProfile {
    email: string;
    displayName: string;
    createdAt: string;
    lastUsed: string;
    note: string;
}

interface RoxyStatus {
    isRunning: boolean;
    pid: number | null;
}

interface AppState {
    users: UserProfile[];
    currentUser: string | null;
    roxyStatus: RoxyStatus;
    isLoading: boolean;
    isSyncing: boolean;
    error: string | null;
    syncStatus: SyncStatus | null;
    snapshots: WebDavSnapshotMeta[];
    wizardOpen: boolean;
    wizardStep: number;
    settingsModalOpen: boolean;

    loadUsers: () => Promise<void>;
    refreshStatus: () => Promise<void>;
    loadSyncStatus: () => Promise<SyncStatus | null>;
    listWebDavSnapshots: () => Promise<WebDavSnapshotMeta[]>;
    syncToWebDav: (silent?: boolean) => Promise<SyncOperationResult>;
    pullLatestFromWebDav: (silent?: boolean) => Promise<SyncOperationResult>;
    restoreWebDavSnapshot: (snapshotId: string, silent?: boolean) => Promise<SyncOperationResult>;
    deleteWebDavSnapshot: (snapshotId: string, silent?: boolean) => Promise<SyncOperationResult>;
    switchUser: (email: string) => Promise<void>;
    deleteUser: (email: string) => Promise<void>;
    startRoxy: () => Promise<void>;
    stopRoxy: () => Promise<void>;
    openWizard: () => void;
    closeWizard: () => void;
    setWizardStep: (step: number) => void;
    prepareForNewUser: () => Promise<void>;
    finalizeNewUser: () => Promise<void>;
    exportProfiles: (path: string) => Promise<string>;
    importProfiles: (path: string) => Promise<string>;
    updateUserNote: (email: string, note: string) => Promise<void>;
    openSettingsModal: () => void;
    closeSettingsModal: () => void;
}

export const useStore = create<AppState>((set, get) => {
    const loadSyncStatusInternal = async (): Promise<SyncStatus | null> => {
        try {
            const status = await invoke<SyncStatus>("get_sync_status");
            set({ syncStatus: status });
            return status;
        } catch (error) {
            console.error("Failed to load sync status:", error);
            return null;
        }
    };

    const runSyncCommand = async (
        command: string,
        args?: Record<string, unknown>,
        silent = false,
    ): Promise<SyncOperationResult> => {
        set({ isSyncing: true, ...(silent ? {} : { error: null }) });
        try {
            const result = await invoke<SyncOperationResult>(command, args);
            await loadSyncStatusInternal();
            return result;
        } catch (error) {
            await loadSyncStatusInternal();
            if (!silent) {
                set({ error: String(error) });
            }
            throw error;
        } finally {
            set({ isSyncing: false });
        }
    };

    const autoSyncIfEnabled = async () => {
        let syncStatus = get().syncStatus;
        if (!syncStatus) {
            syncStatus = await loadSyncStatusInternal();
        }

        if (!syncStatus?.enabled || !syncStatus.autoSyncEnabled) {
            return;
        }

        try {
            await runSyncCommand("sync_to_webdav", undefined, true);
        } catch (error) {
            console.error("Auto WebDAV sync failed:", error);
        }
    };

    return {
        users: [],
        currentUser: null,
        roxyStatus: { isRunning: false, pid: null },
        isLoading: false,
        isSyncing: false,
        error: null,
        syncStatus: null,
        snapshots: [],
        wizardOpen: false,
        wizardStep: 0,
        settingsModalOpen: false,

        loadUsers: async () => {
            try {
                set({ isLoading: true, error: null });
                const result = await invoke<{ users: UserProfile[]; currentUser: string | null }>("list_users");
                set({ users: result.users, currentUser: result.currentUser, isLoading: false });
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        refreshStatus: async () => {
            try {
                const status = await invoke<RoxyStatus>("get_roxy_status");
                set({ roxyStatus: status });
            } catch (error) {
                console.error("Failed to refresh status:", error);
            }
        },

        loadSyncStatus: loadSyncStatusInternal,

        listWebDavSnapshots: async () => {
            try {
                const snapshots = await invoke<WebDavSnapshotMeta[]>("list_webdav_snapshots");
                set({ snapshots });
                return snapshots;
            } catch (error) {
                set({ error: String(error) });
                throw error;
            }
        },

        syncToWebDav: async (silent = false) => runSyncCommand("sync_to_webdav", undefined, silent),

        pullLatestFromWebDav: async (silent = false) => {
            const result = await runSyncCommand("pull_latest_from_webdav", undefined, silent);
            await get().loadUsers();
            return result;
        },

        restoreWebDavSnapshot: async (snapshotId: string, silent = false) => {
            const result = await runSyncCommand("restore_webdav_snapshot", { snapshotId }, silent);
            await get().loadUsers();
            return result;
        },

        deleteWebDavSnapshot: async (snapshotId: string, silent = false) => {
            set({ isSyncing: true, ...(silent ? {} : { error: null }) });
            try {
                const result = await invoke<SyncOperationResult>("delete_webdav_snapshot", { snapshotId });
                await get().listWebDavSnapshots();
                return result;
            } catch (error) {
                if (!silent) {
                    set({ error: String(error) });
                }
                throw error;
            } finally {
                set({ isSyncing: false });
            }
        },

        switchUser: async (email: string) => {
            try {
                set({ isLoading: true, error: null });
                await invoke("switch_user", { email });
                await get().loadUsers();
                await get().refreshStatus();
                set({ isLoading: false });
                await autoSyncIfEnabled();
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        deleteUser: async (email: string) => {
            try {
                set({ isLoading: true, error: null });
                await invoke("delete_user", { email });
                await get().loadUsers();
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        startRoxy: async () => {
            try {
                set({ isLoading: true, error: null });
                await invoke("start_roxy");
                await get().refreshStatus();
                set({ isLoading: false });
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        stopRoxy: async () => {
            try {
                set({ isLoading: true, error: null });
                await invoke("stop_roxy");
                await get().refreshStatus();
                set({ isLoading: false });
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        openWizard: () => set({ wizardOpen: true, wizardStep: 0 }),
        closeWizard: () => set({ wizardOpen: false, wizardStep: 0 }),
        setWizardStep: (step: number) => set({ wizardStep: step }),

        prepareForNewUser: async () => {
            try {
                set({ isLoading: true, error: null });
                await invoke("prepare_for_new_user");
                set({ wizardStep: 1, isLoading: false });
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        finalizeNewUser: async () => {
            try {
                set({ isLoading: true, error: null });
                await invoke("finalize_new_user");
                await get().loadUsers();
                set({ wizardOpen: false, wizardStep: 0, isLoading: false });
                await autoSyncIfEnabled();
            } catch (error) {
                set({ error: String(error), isLoading: false });
            }
        },

        exportProfiles: async (path: string) => {
            try {
                set({ isLoading: true, error: null });
                const result = await invoke<string>("export_profiles", { exportPath: path });
                set({ isLoading: false });
                return result;
            } catch (error) {
                set({ error: String(error), isLoading: false });
                throw error;
            }
        },

        importProfiles: async (path: string) => {
            try {
                set({ isLoading: true, error: null });
                const result = await invoke<string>("import_profiles", { importPath: path });
                await get().loadUsers();
                set({ isLoading: false });
                await autoSyncIfEnabled();
                return result;
            } catch (error) {
                set({ error: String(error), isLoading: false });
                throw error;
            }
        },

        updateUserNote: async (email: string, note: string) => {
            try {
                set({ isLoading: true, error: null });
                await invoke("update_user_note", { email, note });
                await get().loadUsers();
                set({ isLoading: false });
            } catch (error) {
                set({ error: String(error), isLoading: false });
                throw error;
            }
        },

        openSettingsModal: () => set({ settingsModalOpen: true }),
        closeSettingsModal: () => set({ settingsModalOpen: false }),
    };
});
