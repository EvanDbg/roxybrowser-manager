import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

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
    // 状态
    users: UserProfile[];
    currentUser: string | null;
    roxyStatus: RoxyStatus;
    isLoading: boolean;
    error: string | null;

    // 向导状态
    wizardOpen: boolean;
    wizardStep: number;

    // 设置模态框状态
    settingsModalOpen: boolean;

    // Actions
    loadUsers: () => Promise<void>;
    refreshStatus: () => Promise<void>;
    switchUser: (email: string) => Promise<void>;
    deleteUser: (email: string) => Promise<void>;
    startRoxy: () => Promise<void>;
    stopRoxy: () => Promise<void>;

    // 向导 Actions
    openWizard: () => void;
    closeWizard: () => void;
    setWizardStep: (step: number) => void;
    prepareForNewUser: () => Promise<void>;
    finalizeNewUser: () => Promise<void>;

    // 导入导出
    exportProfiles: (path: string) => Promise<string>;
    importProfiles: (path: string) => Promise<string>;

    // 更新备注
    updateUserNote: (email: string, note: string) => Promise<void>;

    // 设置模态框 Actions
    openSettingsModal: () => void;
    closeSettingsModal: () => void;
}

export const useStore = create<AppState>((set, get) => ({
    // 初始状态
    users: [],
    currentUser: null,
    roxyStatus: { isRunning: false, pid: null },
    isLoading: false,
    error: null,
    wizardOpen: false,
    wizardStep: 0,
    settingsModalOpen: false,

    // 加载用户列表
    loadUsers: async () => {
        try {
            set({ isLoading: true, error: null });
            const result = await invoke<{ users: UserProfile[]; currentUser: string | null }>("list_users");
            set({ users: result.users, currentUser: result.currentUser, isLoading: false });
        } catch (error) {
            set({ error: String(error), isLoading: false });
        }
    },

    // 刷新 RoxyBrowser 状态
    refreshStatus: async () => {
        try {
            const status = await invoke<RoxyStatus>("get_roxy_status");
            set({ roxyStatus: status });
        } catch (error) {
            console.error("Failed to refresh status:", error);
        }
    },

    // 切换用户
    switchUser: async (email: string) => {
        try {
            set({ isLoading: true, error: null });
            await invoke("switch_user", { email });
            await get().loadUsers();
            await get().refreshStatus();
        } catch (error) {
            set({ error: String(error), isLoading: false });
        }
    },

    // 删除用户
    deleteUser: async (email: string) => {
        try {
            set({ isLoading: true, error: null });
            await invoke("delete_user", { email });
            await get().loadUsers();
        } catch (error) {
            set({ error: String(error), isLoading: false });
        }
    },

    // 启动 RoxyBrowser
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

    // 停止 RoxyBrowser
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

    // 向导相关
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
        } catch (error) {
            set({ error: String(error), isLoading: false });
        }
    },

    // 导出配置
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

    // 导入配置
    importProfiles: async (path: string) => {
        try {
            set({ isLoading: true, error: null });
            const result = await invoke<string>("import_profiles", { importPath: path });
            await get().loadUsers();
            set({ isLoading: false });
            return result;
        } catch (error) {
            set({ error: String(error), isLoading: false });
            throw error;
        }
    },

    // 更新用户备注
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

    // 设置模态框相关
    openSettingsModal: () => set({ settingsModalOpen: true }),
    closeSettingsModal: () => set({ settingsModalOpen: false }),
}));
