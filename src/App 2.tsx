import { useEffect } from "react";
import Dashboard from "./components/Dashboard";
import { useStore } from "./stores/useStore";

function App() {
    const { loadUsers, refreshStatus } = useStore();

    useEffect(() => {
        // 初始化加载用户列表和状态
        loadUsers();
        refreshStatus();

        // 每 5 秒刷新一次状态
        const interval = setInterval(refreshStatus, 5000);
        return () => clearInterval(interval);
    }, [loadUsers, refreshStatus]);

    return (
        <div className="min-h-screen bg-base-200">
            <Dashboard />
        </div>
    );
}

export default App;
