export interface AppSettings {
    roxyExePath: string | null;
    autoDetectEnabled: boolean;
}

export interface PathValidationResult {
    valid: boolean;
    message?: string;
}
