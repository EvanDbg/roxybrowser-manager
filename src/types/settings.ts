export interface WebDavSettings {
    enabled: boolean;
    baseUrl: string;
    username: string;
    password: string;
    remoteDir: string;
    autoSyncEnabled: boolean;
    lastSyncAt: string | null;
    lastSyncStatus: string | null;
    lastSyncMessage: string | null;
    lastSnapshotId: string | null;
}

export interface AppSettings {
    roxyExePath: string | null;
    autoDetectEnabled: boolean;
    webdav: WebDavSettings;
}

export interface SyncStatus {
    enabled: boolean;
    autoSyncEnabled: boolean;
    lastSyncAt: string | null;
    lastSyncStatus: string | null;
    lastSyncMessage: string | null;
    lastSnapshotId: string | null;
}

export interface SyncOperationResult {
    message: string;
    snapshotId: string | null;
    syncedAt: string | null;
}

export interface WebDavSnapshotMeta {
    id: string;
    createdAt: string;
    userCount: number;
    currentUser: string | null;
    userEmails: string[];
    sourcePlatform: string;
    sourceHost: string;
    sizeBytes: number;
    fileName: string;
}

export interface PathValidationResult {
    valid: boolean;
    message?: string;
}
