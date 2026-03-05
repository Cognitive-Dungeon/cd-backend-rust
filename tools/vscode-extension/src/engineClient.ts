export interface ApiEntity {
    guid: string;
    x: number;
    y: number;
    glyph: string;
    color: string;
}

export interface ApiState {
    tick: number;
    entity_count: number;
    entities: ApiEntity[];
}

export class EngineClient {
    constructor(public baseUrl: string) {}

    get wsUrl(): string {
        return this.baseUrl.replace(/^http/, 'ws') + '/telemetry';
    }

    async fetchState(): Promise<ApiState | null> {
        try {
            const res = await fetch(`${this.baseUrl}/api/state`, { signal: AbortSignal.timeout(1500) });
            return res.ok ? (await res.json() as ApiState) : null;
        } catch {
            return null;
        }
    }

    async hotReload(jsonData: any): Promise<boolean> {
        try {
            const res = await fetch(`${this.baseUrl}/api/reload`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(jsonData)
            });
            return res.ok;
        } catch {
            return false;
        }
    }

    async spawnEntity(guid: string): Promise<boolean> {
        try {
            const res = await fetch(`${this.baseUrl}/api/spawn`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ guid })
            });
            return res.ok;
        } catch {
            return false;
        }
    }
}