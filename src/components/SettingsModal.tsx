import { useState, useEffect } from "react";
import { X, FolderOpen, Search, Trash2, CheckCircle, XCircle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface SettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export default function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
    const [currentPath, setCurrentPath] = useState<string>("");
    const [isValid, setIsValid] = useState<boolean | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string>("");

    // 加载当前配置的路径
    useEffect(() => {
        if (isOpen) {
            loadCurrentPath();
        }
    }, [isOpen]);

    const loadCurrentPath = async () => {
        try {
            const path = await invoke<string | null>("get_roxy_exe_path");
            if (path) {
                setCurrentPath(path);
                await validatePath(path);
            } else {
                setCurrentPath("");
                setIsValid(null);
            }
        } catch (err) {
            console.error("Failed to load path:", err);
        }
    };

    const validatePath = async (path: string) => {
        if (!path.trim()) {
            setIsValid(null);
            return;
        }

        try {
            const valid = await invoke<boolean>("validate_roxy_exe_path", { path });
            setIsValid(valid);
        } catch (err) {
            setIsValid(false);
            console.error("Validation error:", err);
        }
    };

    const handleBrowse = async () => {
        setIsLoading(true);
        setError("");
        try {
            const selectedPath = await invoke<string | null>("browse_for_exe");
            if (selectedPath) {
                setCurrentPath(selectedPath);
                await validatePath(selectedPath);
            }
        } catch (err) {
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    };

    const handleAutoDetect = async () => {
        setIsLoading(true);
        setError("");
        try {
            const detectedPath = await invoke<string | null>("auto_detect_roxy_path");
            if (detectedPath) {
                setCurrentPath(detectedPath);
                setIsValid(true);
            } else {
                setError("未检测到 RoxyBrowser 安装路径，请手动选择");
            }
        } catch (err) {
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    };

    const handleClear = () => {
        setCurrentPath("");
        setIsValid(null);
        setError("");
    };

    const handleSave = async () => {
        if (!currentPath.trim()) {
            // 清除配置
            try {
                await invoke("clear_roxy_exe_path");
                onClose();
            } catch (err) {
                setError(err as string);
            }
            return;
        }

        if (!isValid) {
            setError("请选择有效的 RoxyBrowser 可执行文件");
            return;
        }

        setIsLoading(true);
        setError("");
        try {
            await invoke("set_roxy_exe_path", { path: currentPath });
            onClose();
        } catch (err) {
            setError(err as string);
        } finally {
            setIsLoading(false);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="modal modal-open">
            <div className="modal-box max-w-2xl">
                <button
                    className="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                    onClick={onClose}
                    disabled={isLoading}
                >
                    <X className="w-4 h-4" />
                </button>

                <h3 className="font-bold text-lg mb-4">⚙️ RoxyBrowser 路径配置</h3>

                {/* 当前路径显示 */}
                <div className="mb-4">
                    <label className="label">
                        <span className="label-text">当前路径</span>
                    </label>
                    <div className="flex gap-2 items-center">
                        <input
                            type="text"
                            className="input input-bordered flex-1 font-mono text-sm"
                            value={currentPath}
                            readOnly
                            placeholder="未配置（将使用自动检测）"
                        />
                        {isValid === true && (
                            <div className="tooltip" data-tip="路径有效">
                                <CheckCircle className="w-5 h-5 text-success" />
                            </div>
                        )}
                        {isValid === false && (
                            <div className="tooltip" data-tip="路径无效">
                                <XCircle className="w-5 h-5 text-error" />
                            </div>
                        )}
                    </div>
                </div>

                {/* 操作按钮 */}
                <div className="flex gap-2 mb-4">
                    <button
                        className="btn btn-primary gap-2"
                        onClick={handleBrowse}
                        disabled={isLoading}
                    >
                        <FolderOpen className="w-4 h-4" />
                        浏览...
                    </button>
                    <button
                        className="btn btn-secondary gap-2"
                        onClick={handleAutoDetect}
                        disabled={isLoading}
                    >
                        {isLoading ? (
                            <span className="loading loading-spinner loading-sm"></span>
                        ) : (
                            <Search className="w-4 h-4" />
                        )}
                        自动检测
                    </button>
                    <button
                        className="btn btn-ghost gap-2"
                        onClick={handleClear}
                        disabled={isLoading || !currentPath.trim()}
                    >
                        <Trash2 className="w-4 h-4" />
                        清除
                    </button>
                </div>

                {/* 错误提示 */}
                {error && (
                    <div className="alert alert-error mb-4">
                        <span>{error}</span>
                    </div>
                )}

                {/* 提示信息 */}
                <div className="alert alert-info mb-4">
                    <div className="text-sm">
                        <p className="font-semibold mb-1">💡 使用说明：</p>
                        <ul className="list-disc list-inside space-y-1">
                            <li>点击"浏览"选择 RoxyBrowser.exe 文件</li>
                            <li>点击"自动检测"尝试自动查找安装路径</li>
                            <li>如果不配置，将使用默认的自动检测逻辑</li>
                        </ul>
                    </div>
                </div>

                {/* 底部按钮 */}
                <div className="modal-action">
                    <button
                        className="btn btn-ghost"
                        onClick={onClose}
                        disabled={isLoading}
                    >
                        取消
                    </button>
                    <button
                        className="btn btn-primary"
                        onClick={handleSave}
                        disabled={isLoading || (!!currentPath && !isValid)}
                    >
                        {isLoading ? (
                            <>
                                <span className="loading loading-spinner loading-sm"></span>
                                保存中...
                            </>
                        ) : (
                            "保存"
                        )}
                    </button>
                </div>
            </div>
            <div className="modal-backdrop" onClick={onClose}></div>
        </div>
    );
}
