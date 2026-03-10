import { useEffect, useState } from "react";
import { Clock3, Download, RefreshCw, RotateCcw, Server, Trash2, X } from "lucide-react";
import { useStore } from "../stores/useStore";

interface SyncHistoryModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export default function SyncHistoryModal({ isOpen, onClose }: SyncHistoryModalProps) {
    const {
        isSyncing,
        snapshots,
        listWebDavSnapshots,
        restoreWebDavSnapshot,
        deleteWebDavSnapshot,
        syncStatus,
    } = useStore();
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState("");

    const loadSnapshots = async () => {
        setIsLoading(true);
        setError("");
        try {
            await listWebDavSnapshots();
        } catch (err) {
            setError(String(err));
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        if (isOpen) {
            void loadSnapshots();
        }
    }, [isOpen]);

    const handleRestore = async (snapshotId: string) => {
        const confirmed = confirm(`恢复备份 ${snapshotId} 前，会先创建一份本地安全备份。确定继续吗？`);
        if (!confirmed) {
            return;
        }

        try {
            const result = await restoreWebDavSnapshot(snapshotId);
            alert(result.message);
            onClose();
        } catch (err) {
            alert(`恢复失败: ${err}`);
        }
    };

    const handleDelete = async (snapshotId: string) => {
        const confirmed = confirm(`确定删除远端备份 ${snapshotId} 吗？删除后无法从 WebDAV 恢复该版本。`);
        if (!confirmed) {
            return;
        }

        try {
            const result = await deleteWebDavSnapshot(snapshotId);
            alert(result.message);
            await loadSnapshots();
        } catch (err) {
            alert(`删除失败: ${err}`);
        }
    };

    const formatDateTime = (value: string) => {
        const date = new Date(value);
        if (Number.isNaN(date.getTime())) {
            return value;
        }
        return date.toLocaleString("zh-CN");
    };

    const formatBytes = (value: number) => {
        if (value < 1024) {
            return `${value} B`;
        }
        if (value < 1024 * 1024) {
            return `${(value / 1024).toFixed(1)} KB`;
        }
        if (value < 1024 * 1024 * 1024) {
            return `${(value / 1024 / 1024).toFixed(1)} MB`;
        }
        return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB`;
    };

    if (!isOpen) {
        return null;
    }

    return (
        <div className="modal modal-open">
            <div className="modal-box max-w-4xl">
                <button
                    className="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                    onClick={onClose}
                    disabled={isLoading || isSyncing}
                >
                    <X className="w-4 h-4" />
                </button>

                <div className="flex items-center justify-between gap-3 mb-4">
                    <div>
                        <h3 className="font-bold text-lg">🗂️ WebDAV 备份数据管理</h3>
                        <p className="text-sm text-base-content/60 mt-1">
                            这里显示的是每份备份的摘要信息；实际登录数据保存在对应压缩包里。
                        </p>
                    </div>
                    <button
                        className="btn btn-ghost btn-sm gap-2"
                        onClick={loadSnapshots}
                        disabled={isLoading || isSyncing}
                    >
                        <RefreshCw className={`w-4 h-4 ${isLoading ? "animate-spin" : ""}`} />
                        刷新
                    </button>
                </div>

                {!syncStatus?.enabled && (
                    <div className="alert alert-warning mb-4">
                        <span>尚未启用 WebDAV 备份，请先返回上一级完成配置。</span>
                    </div>
                )}

                {syncStatus?.lastSyncMessage && (
                    <div className={`alert mb-4 ${syncStatus.lastSyncStatus === "error" ? "alert-error" : "alert-info"}`}>
                        <span>{syncStatus.lastSyncMessage}</span>
                    </div>
                )}

                <div className="alert alert-info mb-4">
                    <span>每份备份都包含当时本地的全部账号信息；恢复时会整体替换本地账号数据。</span>
                </div>

                {error && (
                    <div className="alert alert-error mb-4">
                        <span>{error}</span>
                    </div>
                )}

                <div className="space-y-3 max-h-[60vh] overflow-y-auto pr-1">
                    {!isLoading && snapshots.length === 0 && (
                        <div className="card bg-base-200 border border-base-300">
                            <div className="card-body items-center text-center py-8">
                                <Clock3 className="w-10 h-10 text-base-content/30" />
                                <p className="text-base-content/60">WebDAV 中还没有可恢复的备份</p>
                            </div>
                        </div>
                    )}

                    {snapshots.map((snapshot) => (
                        <div key={snapshot.id} className="card bg-base-200 border border-base-300">
                            <div className="card-body p-4 gap-3">
                                <div className="flex items-start justify-between gap-4 flex-wrap">
                                    <div>
                                        <div className="font-semibold flex items-center gap-2">
                                            <Clock3 className="w-4 h-4 text-primary" />
                                            {formatDateTime(snapshot.createdAt)}
                                        </div>
                                        <div className="text-sm text-base-content/60 mt-1">
                                            备份 ID: <span className="font-mono">{snapshot.id}</span>
                                        </div>
                                    </div>
                                    <div className="flex items-center gap-2 flex-wrap">
                                        <button
                                            className="btn btn-primary btn-sm gap-2"
                                            onClick={() => handleRestore(snapshot.id)}
                                            disabled={isSyncing}
                                        >
                                            <RotateCcw className="w-4 h-4" />
                                            恢复
                                        </button>
                                        <button
                                            className="btn btn-ghost btn-sm gap-2 text-error"
                                            onClick={() => handleDelete(snapshot.id)}
                                            disabled={isSyncing}
                                        >
                                            <Trash2 className="w-4 h-4" />
                                            删除
                                        </button>
                                    </div>
                                </div>

                                <div className="grid grid-cols-1 md:grid-cols-4 gap-2 text-sm">
                                    <div className="badge badge-outline h-auto py-2 justify-start">
                                        账号数: {snapshot.userCount}
                                    </div>
                                    <div className="badge badge-outline h-auto py-2 justify-start">
                                        当前账号: {snapshot.currentUser ?? "无"}
                                    </div>
                                    <div className="badge badge-outline h-auto py-2 justify-start">
                                        大小: {formatBytes(snapshot.sizeBytes)}
                                    </div>
                                    <div className="badge badge-outline h-auto py-2 justify-start gap-1">
                                        <Server className="w-3.5 h-3.5" />
                                        {snapshot.sourceHost} / {snapshot.sourcePlatform}
                                    </div>
                                </div>

                                {snapshot.userEmails.length > 0 && (
                                    <div className="flex flex-wrap gap-2">
                                        {snapshot.userEmails.map((email) => (
                                            <span key={email} className="badge badge-primary badge-outline">
                                                {email}
                                            </span>
                                        ))}
                                    </div>
                                )}

                                <div className="text-xs text-base-content/50 flex items-center gap-2">
                                    <Download className="w-3.5 h-3.5" />
                                    {snapshot.fileName}
                                </div>
                            </div>
                        </div>
                    ))}
                </div>

                <div className="modal-action">
                    <button className="btn btn-ghost" onClick={onClose} disabled={isLoading || isSyncing}>
                        关闭
                    </button>
                </div>
            </div>
            <div className="modal-backdrop" onClick={onClose}></div>
        </div>
    );
}
