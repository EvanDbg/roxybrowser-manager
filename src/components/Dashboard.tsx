import { User, Play, Square, RefreshCw, Plus, Download, Upload, Settings, Sun, Moon, X } from "lucide-react";
import { useStore } from "../stores/useStore";
import UserCard from "./UserCard";
import AddUserWizard from "./AddUserWizard";
import { useEffect, useState, useRef } from "react";

export default function Dashboard() {
    const {
        users,
        currentUser,
        roxyStatus,
        isLoading,
        error,
        startRoxy,
        stopRoxy,
        refreshStatus,
        openWizard,
        wizardOpen,
        exportProfiles,
        importProfiles,
    } = useStore();

    // 主题状态
    const [theme, setTheme] = useState<'light' | 'dark'>(() => {
        const saved = localStorage.getItem('theme');
        return (saved as 'light' | 'dark') || 'dark';
    });

    // 导入导出模态框状态
    const [modalType, setModalType] = useState<'export' | 'import' | null>(null);
    const [pathValue, setPathValue] = useState("~/Desktop/roxy_manager_backup");
    const inputRef = useRef<HTMLInputElement>(null);

    // 应用主题
    useEffect(() => {
        document.documentElement.setAttribute('data-theme', theme);
        localStorage.setItem('theme', theme);
    }, [theme]);

    const toggleTheme = () => {
        setTheme(prev => prev === 'dark' ? 'light' : 'dark');
    };

    // 快捷键支持
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.metaKey || e.ctrlKey) {
                switch (e.key) {
                    case 'n':
                        e.preventDefault();
                        openWizard();
                        break;
                    case 'r':
                        e.preventDefault();
                        refreshStatus();
                        break;
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [openWizard, refreshStatus]);

    // 打开模态框
    const openExportModal = () => {
        setPathValue("~/Desktop/roxy_manager_backup");
        setModalType('export');
        setTimeout(() => inputRef.current?.focus(), 100);
    };

    const openImportModal = () => {
        setPathValue("~/Desktop/roxy_manager_backup");
        setModalType('import');
        setTimeout(() => inputRef.current?.focus(), 100);
    };

    const closeModal = () => {
        setModalType(null);
    };

    // 执行导出
    const handleExport = async () => {
        if (!pathValue.trim()) return;
        try {
            const result = await exportProfiles(pathValue.trim());
            closeModal();
            alert(result);
        } catch (error) {
            alert(`导出失败: ${error}`);
        }
    };

    // 执行导入
    const handleImport = async () => {
        if (!pathValue.trim()) return;
        try {
            const result = await importProfiles(pathValue.trim());
            closeModal();
            alert(result);
        } catch (error) {
            alert(`导入失败: ${error}`);
        }
    };

    return (
        <div className="container mx-auto p-6 max-w-2xl">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="text-4xl">🦊</div>
                    <div>
                        <h1 className="text-2xl font-bold">RoxyBrowser Manager</h1>
                        <p className="text-base-content/60 text-sm">多账户快速切换工具</p>
                    </div>
                </div>
                <div className="flex items-center gap-2">
                    {/* 主题切换按钮 */}
                    <button
                        className="btn btn-ghost btn-sm btn-circle"
                        onClick={toggleTheme}
                        title={theme === 'dark' ? '切换到浅色模式' : '切换到深色模式'}
                    >
                        {theme === 'dark' ? <Sun className="w-5 h-5" /> : <Moon className="w-5 h-5" />}
                    </button>
                    {/* 设置下拉菜单 */}
                    <div className="dropdown dropdown-end">
                        <div tabIndex={0} role="button" className="btn btn-ghost btn-sm">
                            <Settings className="w-5 h-5" />
                        </div>
                        <ul tabIndex={0} className="dropdown-content menu bg-base-200 rounded-box z-50 w-48 p-2 shadow-lg">
                            <li>
                                <a
                                    onClick={(e) => {
                                        e.preventDefault();
                                        (document.activeElement as HTMLElement)?.blur();
                                        openExportModal();
                                    }}
                                    className={isLoading || users.length === 0 ? 'disabled' : ''}
                                >
                                    <Download className="w-4 h-4" />
                                    导出配置
                                </a>
                            </li>
                            <li>
                                <a
                                    onClick={(e) => {
                                        e.preventDefault();
                                        (document.activeElement as HTMLElement)?.blur();
                                        openImportModal();
                                    }}
                                    className={isLoading ? 'disabled' : ''}
                                >
                                    <Upload className="w-4 h-4" />
                                    导入配置
                                </a>
                            </li>
                        </ul>
                    </div>
                </div>
            </div>

            {/* 错误提示 */}
            {error && (
                <div className="alert alert-error mb-4">
                    <span>{error}</span>
                </div>
            )}

            {/* 状态卡片 */}
            <div className="card bg-base-100 shadow-lg mb-6">
                <div className="card-body p-4">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className={`badge ${roxyStatus.isRunning ? 'badge-success' : 'badge-ghost'} gap-2`}>
                                <span className={`w-2 h-2 rounded-full ${roxyStatus.isRunning ? 'bg-success animate-pulse' : 'bg-base-content/30'}`} />
                                {roxyStatus.isRunning ? 'RoxyBrowser 运行中' : 'RoxyBrowser 已停止'}
                            </div>
                        </div>
                        <div className="flex items-center gap-2">
                            <button
                                className="btn btn-ghost btn-sm gap-1"
                                onClick={refreshStatus}
                                disabled={isLoading}
                            >
                                <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
                                刷新
                            </button>
                            <button
                                className="btn btn-primary btn-sm gap-1"
                                onClick={openWizard}
                                disabled={isLoading}
                            >
                                <Plus className="w-4 h-4" />
                                添加用户
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            {/* 用户列表 */}
            <div className="space-y-3">
                {users.length === 0 ? (
                    <div className="card bg-base-100 shadow">
                        <div className="card-body items-center text-center py-8">
                            <User className="w-12 h-12 text-base-content/30" />
                            <p className="text-base-content/60">暂无用户</p>
                            <p className="text-sm text-base-content/40">点击上方"添加用户"按钮添加第一个用户</p>
                        </div>
                    </div>
                ) : (
                    users.map((user) => (
                        <UserCard
                            key={user.email}
                            user={user}
                            isActive={user.email === currentUser}
                        />
                    ))
                )}
            </div>

            {/* 操作按钮 */}
            <div className="flex gap-3 mt-6">
                <button
                    className="btn btn-success flex-1 gap-2"
                    onClick={startRoxy}
                    disabled={isLoading || roxyStatus.isRunning}
                >
                    <Play className="w-4 h-4" />
                    启动
                </button>
                <button
                    className="btn btn-error flex-1 gap-2"
                    onClick={stopRoxy}
                    disabled={isLoading || !roxyStatus.isRunning}
                >
                    <Square className="w-4 h-4" />
                    停止
                </button>
            </div>

            {/* 快捷键提示 */}
            <div className="mt-4 text-center text-xs text-base-content/40">
                快捷键: ⌘N 添加用户 | ⌘R 刷新状态
            </div>

            {/* 添加用户向导 */}
            {wizardOpen && <AddUserWizard />}

            {/* 导入导出模态框 */}
            {modalType && (
                <div className="modal modal-open">
                    <div className="modal-box">
                        <button
                            className="btn btn-sm btn-circle btn-ghost absolute right-2 top-2"
                            onClick={closeModal}
                        >
                            <X className="w-4 h-4" />
                        </button>
                        <h3 className="font-bold text-lg mb-4">
                            {modalType === 'export' ? '📤 导出配置' : '📥 导入配置'}
                        </h3>
                        <p className="text-sm text-base-content/60 mb-3">
                            {modalType === 'export'
                                ? '请输入导出目录路径，配置将保存到该目录：'
                                : '请输入要导入的配置目录路径：'}
                        </p>
                        <input
                            ref={inputRef}
                            type="text"
                            className="input input-bordered w-full"
                            value={pathValue}
                            onChange={(e) => setPathValue(e.target.value)}
                            placeholder="例如: ~/Desktop/roxy_manager_backup"
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    modalType === 'export' ? handleExport() : handleImport();
                                }
                                if (e.key === 'Escape') closeModal();
                            }}
                        />
                        <div className="modal-action">
                            <button className="btn btn-ghost" onClick={closeModal}>
                                取消
                            </button>
                            <button
                                className="btn btn-primary"
                                onClick={modalType === 'export' ? handleExport : handleImport}
                                disabled={!pathValue.trim() || isLoading}
                            >
                                {modalType === 'export' ? '导出' : '导入'}
                            </button>
                        </div>
                    </div>
                    <div className="modal-backdrop" onClick={closeModal}></div>
                </div>
            )}
        </div>
    );
}
