import { AlertTriangle, X, ArrowLeft, ArrowRight, Check, Loader2 } from "lucide-react";
import { useStore } from "../stores/useStore";

export default function AddUserWizard() {
    const {
        wizardStep,
        isLoading,
        error,
        closeWizard,
        setWizardStep,
        prepareForNewUser,
        finalizeNewUser,
    } = useStore();

    const steps = [
        { title: "准备", description: "保存当前用户并准备登录新账户" },
        { title: "登录", description: "在 RoxyBrowser 中登录新账户" },
        { title: "完成", description: "保存新用户配置" },
    ];

    const handleStart = async () => {
        await prepareForNewUser();
    };

    const handleFinish = async () => {
        await finalizeNewUser();
    };

    return (
        <div className="modal modal-open">
            <div className="modal-box">
                {/* Header */}
                <div className="flex items-center justify-between mb-4">
                    <h3 className="font-bold text-lg">
                        添加新用户 - 步骤 {wizardStep + 1}/{steps.length}
                    </h3>
                    <button className="btn btn-sm btn-circle btn-ghost" onClick={closeWizard}>
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Steps indicator */}
                <ul className="steps steps-horizontal w-full mb-6">
                    {steps.map((step, index) => (
                        <li
                            key={index}
                            className={`step ${index <= wizardStep ? "step-primary" : ""}`}
                        >
                            {step.title}
                        </li>
                    ))}
                </ul>

                {/* Error */}
                {error && (
                    <div className="alert alert-error mb-4">
                        <span>{error}</span>
                    </div>
                )}

                {/* Step Content */}
                <div className="min-h-[200px]">
                    {wizardStep === 0 && (
                        <div className="space-y-4">
                            <div className="alert alert-warning">
                                <AlertTriangle className="w-5 h-5" />
                                <div>
                                    <h4 className="font-bold">重要提示</h4>
                                    <ul className="text-sm mt-2 space-y-1">
                                        <li>1. 将自动保存当前用户配置</li>
                                        <li>2. 清空 RoxyBrowser 登录状态</li>
                                        <li>3. 启动 RoxyBrowser 供您登录新账户</li>
                                    </ul>
                                </div>
                            </div>
                            <div className="alert alert-error">
                                <AlertTriangle className="w-5 h-5" />
                                <div>
                                    <p className="font-bold">登录成功后，请直接关闭窗口或点击下方完成</p>
                                    <p className="text-sm">切勿在 RoxyBrowser 中点击"退出登录"！</p>
                                </div>
                            </div>
                        </div>
                    )}

                    {wizardStep === 1 && (
                        <div className="text-center space-y-4 py-4">
                            <div className="text-6xl mb-4">🔐</div>
                            <h4 className="text-xl font-bold">请在 RoxyBrowser 中登录新账户</h4>
                            <p className="text-base-content/60">
                                RoxyBrowser 已启动，请完成登录操作
                            </p>
                            <p className="text-sm text-warning">
                                登录完成后，点击下方"完成登录"按钮
                            </p>
                        </div>
                    )}

                    {wizardStep === 2 && (
                        <div className="text-center space-y-4 py-4">
                            <div className="text-6xl mb-4">✅</div>
                            <h4 className="text-xl font-bold">检测并保存新用户</h4>
                            <p className="text-base-content/60">
                                点击下方按钮完成配置保存
                            </p>
                        </div>
                    )}
                </div>

                {/* Actions */}
                <div className="modal-action">
                    {wizardStep === 0 && (
                        <>
                            <button className="btn btn-ghost" onClick={closeWizard}>
                                取消
                            </button>
                            <button
                                className="btn btn-primary gap-2"
                                onClick={handleStart}
                                disabled={isLoading}
                            >
                                {isLoading ? (
                                    <Loader2 className="w-4 h-4 animate-spin" />
                                ) : (
                                    <ArrowRight className="w-4 h-4" />
                                )}
                                开始添加
                            </button>
                        </>
                    )}

                    {wizardStep === 1 && (
                        <>
                            <button
                                className="btn btn-ghost gap-2"
                                onClick={() => setWizardStep(0)}
                                disabled={isLoading}
                            >
                                <ArrowLeft className="w-4 h-4" />
                                返回
                            </button>
                            <button
                                className="btn btn-success gap-2"
                                onClick={() => setWizardStep(2)}
                                disabled={isLoading}
                            >
                                <Check className="w-4 h-4" />
                                完成登录
                            </button>
                        </>
                    )}

                    {wizardStep === 2 && (
                        <>
                            <button
                                className="btn btn-ghost gap-2"
                                onClick={() => setWizardStep(1)}
                                disabled={isLoading}
                            >
                                <ArrowLeft className="w-4 h-4" />
                                返回
                            </button>
                            <button
                                className="btn btn-primary gap-2"
                                onClick={handleFinish}
                                disabled={isLoading}
                            >
                                {isLoading ? (
                                    <Loader2 className="w-4 h-4 animate-spin" />
                                ) : (
                                    <Check className="w-4 h-4" />
                                )}
                                保存用户
                            </button>
                        </>
                    )}
                </div>
            </div>
            <div className="modal-backdrop bg-black/50" onClick={closeWizard} />
        </div>
    );
}
