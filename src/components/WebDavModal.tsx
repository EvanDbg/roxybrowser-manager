import { useEffect, useMemo, useState } from "react";
import {
    Cloud,
    CloudUpload,
    FolderTree,
    RefreshCw,
    ShieldCheck,
    Upload,
    X,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { WebDavSettings } from "../types/settings";
import { useStore } from "../stores/useStore";
import SyncHistoryModal from "./SyncHistoryModal";

interface WebDavModalProps {
    isOpen: boolean;
    onClose: () => void;
}

const defaultWebDavSettings: WebDavSettings = {
    enabled: false,
    baseUrl: "",
    username: "",
    password: "",
    remoteDir: "roxybrowser-manager",
    autoSyncEnabled: true,
    lastSyncAt: null,
    lastSyncStatus: null,
    lastSyncMessage: null,
    lastSnapshotId: null,
};

export default function WebDavModal({ isOpen, onClose }: WebDavModalProps) {
    const { loadSyncStatus, syncStatus, isSyncing, syncToWebDav } = useStore();
    const [webdavSettings, setWebdavSettings] = useState<WebDavSettings>(defaultWebDavSettings);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState("");
    const [message, setMessage] = useState("");
    const [restoreModalOpen, setRestoreModalOpen] = useState(false);

    useEffect(() => {
        if (isOpen) {
            void loadSettings();
        }
    }, [isOpen]);

    const statusMessage = useMemo(
        () => syncStatus?.lastSyncMessage ?? webdavSettings.lastSyncMessage,
        [syncStatus, webdavSettings.lastSyncMessage],
    );

    const feedbackTone = message
        ? "alert-success"
        : (syncStatus?.lastSyncStatus ?? webdavSettings.lastSyncStatus) === "error"
          ? "alert-error"
          : "alert-success";

    const updateWebDav = <K extends keyof WebDavSettings>(key: K, value: WebDavSettings[K]) => {
        setWebdavSettings((prev) => ({ ...prev, [key]: value }));
    };

    const loadSettings = async () => {
        setError("");
        setMessage("");
        try {
            const [savedWebDavSettings] = await Promise.all([
                invoke<WebDavSettings>("get_webdav_settings"),
                loadSyncStatus(),
            ]);
            setWebdavSettings({ ...defaultWebDavSettings, ...savedWebDavSettings });
        } catch (err) {
            setError(String(err));
        }
    };

    const persistSettings = async () => {
        const saved = await invoke<WebDavSettings>("save_webdav_settings", {
            settings: {
                enabled: webdavSettings.enabled,
                baseUrl: webdavSettings.baseUrl,
                username: webdavSettings.username,
                password: webdavSettings.password,
                remoteDir: webdavSettings.remoteDir,
                autoSyncEnabled: webdavSettings.autoSyncEnabled,
            },
        });
        setWebdavSettings((prev) => ({ ...prev, ...saved }));
        await loadSyncStatus();
        return saved;
    };

    const formatDateTime = (value: string | null) => {
        if (!value) {
            return "尚未备份";
        }
        const date = new Date(value);
        if (Number.isNaN(date.getTime())) {
            return value;
        }
        return date.toLocaleString("zh-CN");
    };

    const handleSave = async () => {
        setIsLoading(true);
        setError("");
        setMessage("");
        try {
            await persistSettings();
            setMessage("WebDAV 配置已保存");
        } catch (err) {
            setError(String(err));
        } finally {
            setIsLoading(false);
        }
    };

    const handleTest = async () => {
        setIsLoading(true);
        setError("");
        setMessage("");
        try {
            const result = await invoke<string>("test_webdav_connection", {
                settings: {
                    enabled: webdavSettings.enabled,
                    baseUrl: webdavSettings.baseUrl,
                    username: webdavSettings.username,
                    password: webdavSettings.password,
                    remoteDir: webdavSettings.remoteDir,
                    autoSyncEnabled: webdavSettings.autoSyncEnabled,
                },
            });
            setMessage(result);
        } catch (err) {
            setError(String(err));
        } finally {
            setIsLoading(false);
        }
    };

    const ensureReadyForBackup = async () => {
        const saved = await persistSettings();
        if (!saved.enabled) {
            throw new Error("请先启用 WebDAV 备份");
        }
    };

    const handleBackup = async () => {
        setIsLoading(true);
        setError("");
        setMessage("");
        try {
            await ensureReadyForBackup();
            const result = await syncToWebDav();
            setMessage(result.message);
        } catch (err) {
            setError(String(err));
        } finally {
            setIsLoading(false);
        }
    };

    const handleOpenRestore = async () => {
        setIsLoading(true);
        setError("");
        setMessage("");
        try {
            await ensureReadyForBackup();
            setRestoreModalOpen(true);
        } catch (err) {
            setError(String(err));
        } finally {
            setIsLoading(false);
        }
    };

    if (!isOpen) {
        return null;
    }

    return (
        <>
            <div className="modal modal-open">
                <div className="modal-box max-w-4xl">
                    <button
                        className="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                        onClick={onClose}
                        disabled={isLoading || isSyncing}
                    >
                        <X className="w-4 h-4" />
                    </button>

                    <div className="mb-5">
                        <h3 className="font-bold text-lg">☁️ WebDAV</h3>
                        <p className="text-sm text-base-content/60 mt-1">
                            用于备份与恢复本地全部账号数据，首页不展示，按需在这里使用。
                        </p>
                    </div>

                    <div className="space-y-5 max-h-[70vh] overflow-y-auto pr-1">
                        <div className="card bg-base-200 border border-base-300">
                            <div className="card-body p-4 gap-4">
                                <div className="flex items-center justify-between gap-4 flex-wrap">
                                    <div>
                                        <h4 className="font-semibold flex items-center gap-2">
                                            <Cloud className="w-4 h-4 text-primary" />
                                            基础配置
                                        </h4>
                                        <p className="text-sm text-base-content/60 mt-1">
                                            支持标准 WebDAV 地址、用户名、密码和路径配置。
                                        </p>
                                    </div>
                                    <label className="label cursor-pointer gap-3">
                                        <span className="label-text">启用 WebDAV</span>
                                        <input
                                            type="checkbox"
                                            className="toggle toggle-primary"
                                            checked={webdavSettings.enabled}
                                            onChange={(e) => updateWebDav("enabled", e.target.checked)}
                                            disabled={isLoading || isSyncing}
                                        />
                                    </label>
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <label className="form-control">
                                        <span className="label-text mb-2">WebDAV 地址</span>
                                        <input
                                            type="url"
                                            className="input input-bordered"
                                            value={webdavSettings.baseUrl}
                                            onChange={(e) => updateWebDav("baseUrl", e.target.value)}
                                            placeholder="https://dav.example.com/webdav"
                                            disabled={isLoading || isSyncing}
                                        />
                                    </label>
                                    <label className="form-control">
                                        <span className="label-text mb-2">路径</span>
                                        <input
                                            type="text"
                                            className="input input-bordered"
                                            value={webdavSettings.remoteDir}
                                            onChange={(e) => updateWebDav("remoteDir", e.target.value)}
                                            placeholder="roxybrowser-manager"
                                            disabled={isLoading || isSyncing}
                                        />
                                    </label>
                                    <label className="form-control">
                                        <span className="label-text mb-2">用户名</span>
                                        <input
                                            type="text"
                                            className="input input-bordered"
                                            value={webdavSettings.username}
                                            onChange={(e) => updateWebDav("username", e.target.value)}
                                            placeholder="WebDAV 用户名"
                                            disabled={isLoading || isSyncing}
                                        />
                                    </label>
                                    <label className="form-control">
                                        <span className="label-text mb-2">密码</span>
                                        <input
                                            type="password"
                                            className="input input-bordered"
                                            value={webdavSettings.password}
                                            onChange={(e) => updateWebDav("password", e.target.value)}
                                            placeholder="WebDAV 密码"
                                            disabled={isLoading || isSyncing}
                                        />
                                    </label>
                                </div>

                                <div className="flex items-center justify-between gap-4 flex-wrap rounded-box bg-base-100 px-4 py-3">
                                    <div>
                                        <div className="font-medium">关键操作自动备份</div>
                                        <div className="text-sm text-base-content/60">
                                            切换用户、完成新用户添加、导入配置成功后自动上传一份完整备份。
                                        </div>
                                    </div>
                                    <input
                                        type="checkbox"
                                        className="toggle toggle-primary"
                                        checked={webdavSettings.autoSyncEnabled}
                                        onChange={(e) => updateWebDav("autoSyncEnabled", e.target.checked)}
                                        disabled={isLoading || isSyncing}
                                    />
                                </div>

                                <div className="flex gap-2 flex-wrap">
                                    <button
                                        className="btn btn-outline gap-2"
                                        onClick={handleTest}
                                        disabled={isLoading || isSyncing}
                                    >
                                        <RefreshCw className="w-4 h-4" />
                                        测试连接
                                    </button>
                                    <button
                                        className="btn btn-primary gap-2"
                                        onClick={handleSave}
                                        disabled={isLoading || isSyncing}
                                    >
                                        {isLoading ? (
                                            <span className="loading loading-spinner loading-sm"></span>
                                        ) : (
                                            <Upload className="w-4 h-4" />
                                        )}
                                        保存配置
                                    </button>
                                </div>
                            </div>
                        </div>

                        <div className="card bg-base-200 border border-base-300">
                            <div className="card-body p-4 gap-4">
                                <div className="flex items-center justify-between gap-4 flex-wrap">
                                    <div>
                                        <h4 className="font-semibold flex items-center gap-2">
                                            <FolderTree className="w-4 h-4 text-primary" />
                                            数据备份与恢复
                                        </h4>
                                        <p className="text-sm text-base-content/60 mt-1">
                                            每次都会备份本地全部账号信息；恢复时会先创建本地安全备份。
                                        </p>
                                    </div>
                                    <div className="flex gap-2 flex-wrap">
                                        <button
                                            className="btn btn-primary gap-2"
                                            onClick={handleBackup}
                                            disabled={isLoading || isSyncing}
                                        >
                                            <CloudUpload className={`w-4 h-4 ${isSyncing ? "animate-pulse" : ""}`} />
                                            备份到 WebDAV
                                        </button>
                                        <button
                                            className="btn btn-outline gap-2"
                                            onClick={handleOpenRestore}
                                            disabled={isLoading || isSyncing}
                                        >
                                            <FolderTree className="w-4 h-4" />
                                            从 WebDAV 恢复
                                        </button>
                                    </div>
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm">
                                    <div className="badge badge-outline h-auto py-2 justify-start">
                                        最近备份: {formatDateTime(syncStatus?.lastSyncAt ?? webdavSettings.lastSyncAt)}
                                    </div>
                                    <div className="badge badge-outline h-auto py-2 justify-start">
                                        最近状态: {syncStatus?.lastSyncStatus ?? webdavSettings.lastSyncStatus ?? "idle"}
                                    </div>
                                    <div className="badge badge-outline h-auto py-2 justify-start">
                                        最近备份 ID: {syncStatus?.lastSnapshotId ?? webdavSettings.lastSnapshotId ?? "无"}
                                    </div>
                                </div>

                                <div className="alert alert-info">
                                    <ShieldCheck className="w-5 h-5" />
                                    <span>
                                        列表里显示的是备份摘要；真正可用于切号与恢复登录的数据在对应压缩包中，且包含
                                        `config.json + profiles/*` 的完整账号备份。
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>

                    {error && (
                        <div className="alert alert-error mt-4">
                            <span>{error}</span>
                        </div>
                    )}

                    {!error && (message || statusMessage) && (
                        <div className={`alert mt-4 ${feedbackTone}`}>
                            <span>{message || statusMessage}</span>
                        </div>
                    )}

                    <div className="modal-action">
                        <button className="btn btn-ghost" onClick={onClose} disabled={isLoading || isSyncing}>
                            关闭
                        </button>
                    </div>
                </div>
                <div className="modal-backdrop" onClick={onClose}></div>
            </div>

            <SyncHistoryModal isOpen={restoreModalOpen} onClose={() => setRestoreModalOpen(false)} />
        </>
    );
}
