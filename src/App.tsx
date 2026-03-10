import { useEffect } from "react";
import Dashboard from "./components/Dashboard";
import { useStore } from "./stores/useStore";

function App() {
    const { loadUsers, refreshStatus, loadSyncStatus } = useStore();

    useEffect(() => {
        loadUsers();
        refreshStatus();
        loadSyncStatus();

        const interval = setInterval(refreshStatus, 5000);
        return () => clearInterval(interval);
    }, [loadUsers, refreshStatus, loadSyncStatus]);

    return (
        <div className="min-h-screen bg-base-200">
            <Dashboard />
        </div>
    );
}

export default App;
