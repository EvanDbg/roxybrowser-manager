import { User, ArrowRightCircle, Trash2, Clock, Edit3, Check, X } from "lucide-react";
import { useStore, type UserProfile } from "../stores/useStore";
import { useState } from "react";

interface UserCardProps {
    user: UserProfile;
    isActive: boolean;
}

export default function UserCard({ user, isActive }: UserCardProps) {
    const { switchUser, deleteUser, updateUserNote, isLoading } = useStore();
    const [isEditingNote, setIsEditingNote] = useState(false);
    const [noteValue, setNoteValue] = useState(user.note || "");

    const handleSwitch = () => {
        if (!isActive) {
            switchUser(user.email);
        }
    };

    const handleDelete = () => {
        if (confirm(`确定要删除用户 ${user.email} 吗？\n\n此操作将删除该用户的本地配置备份。`)) {
            deleteUser(user.email);
        }
    };

    const handleEditNote = () => {
        setNoteValue(user.note || "");
        setIsEditingNote(true);
    };

    const handleSaveNote = async () => {
        try {
            await updateUserNote(user.email, noteValue);
            setIsEditingNote(false);
        } catch (e) {
            console.error("Error saving note:", e);
        }
    };

    const handleCancelEdit = () => {
        setIsEditingNote(false);
        setNoteValue(user.note || "");
    };

    const formatDate = (dateStr: string) => {
        try {
            const date = new Date(dateStr);
            return date.toLocaleString('zh-CN', {
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
            });
        } catch {
            return dateStr;
        }
    };

    return (
        <div className={`card bg-base-100 shadow ${isActive ? 'ring-2 ring-primary' : ''}`}>
            <div className="card-body p-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <div className={`avatar placeholder ${isActive ? 'online' : ''}`}>
                            <div className={`w-10 rounded-full ${isActive ? 'bg-primary text-primary-content' : 'bg-base-300'}`}>
                                <User className="w-5 h-5" />
                            </div>
                        </div>
                        <div>
                            <div className="flex items-center gap-2">
                                <span className="font-medium">{user.displayName || user.email}</span>
                                {isActive && (
                                    <span className="badge badge-primary badge-sm">当前</span>
                                )}
                            </div>
                            <div className="text-sm text-base-content/60">{user.email}</div>
                        </div>
                    </div>

                    <div className="flex items-center gap-2">
                        {!isActive && (
                            <button
                                className="btn btn-primary btn-sm gap-1"
                                onClick={handleSwitch}
                                disabled={isLoading}
                            >
                                <ArrowRightCircle className="w-4 h-4" />
                                切换
                            </button>
                        )}
                        <button
                            className="btn btn-ghost btn-sm"
                            onClick={handleEditNote}
                            disabled={isLoading || isEditingNote}
                            title="编辑备注"
                        >
                            <Edit3 className="w-4 h-4" />
                        </button>
                        <button
                            className="btn btn-ghost btn-sm text-error"
                            onClick={handleDelete}
                            disabled={isLoading || isActive}
                            title={isActive ? "无法删除当前用户" : "删除用户"}
                        >
                            <Trash2 className="w-4 h-4" />
                        </button>
                    </div>
                </div>

                {/* 备注编辑区域 */}
                {isEditingNote ? (
                    <div className="mt-2 flex items-center gap-2">
                        <input
                            type="text"
                            className="input input-bordered input-sm flex-1"
                            placeholder="输入备注..."
                            value={noteValue}
                            onChange={(e) => setNoteValue(e.target.value)}
                            autoFocus
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') handleSaveNote();
                                if (e.key === 'Escape') handleCancelEdit();
                            }}
                        />
                        <button className="btn btn-success btn-sm btn-square" onClick={handleSaveNote}>
                            <Check className="w-4 h-4" />
                        </button>
                        <button className="btn btn-ghost btn-sm btn-square" onClick={handleCancelEdit}>
                            <X className="w-4 h-4" />
                        </button>
                    </div>
                ) : user.note ? (
                    <div className="mt-2 px-3 py-2 bg-base-200 rounded-lg text-sm">
                        📝 {user.note}
                    </div>
                ) : null}

                <div className="flex items-center gap-1 text-xs text-base-content/50 mt-2">
                    <Clock className="w-3 h-3" />
                    <span>最后使用: {formatDate(user.lastUsed)}</span>
                </div>
            </div>
        </div>
    );
}
