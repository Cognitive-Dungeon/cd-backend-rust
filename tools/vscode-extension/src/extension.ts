import * as vscode from 'vscode';

// ──────────────────────────────── Types ───────────────────────────────────

interface ApiEntity {
    guid: string; x: number; y: number;
    glyph: string; color: string;
}
interface ApiState {
    tick: number; entity_count: number; entities: ApiEntity[];
}

// ──────────────────────────────── EngineClient ────────────────────────────

class EngineClient {
    constructor(private baseUrl: string) {}

    async fetchState(): Promise<ApiState | null> {
        try {
            const res = await fetch(`${this.baseUrl}/api/state`,
                { signal: AbortSignal.timeout(1500) });
            return res.ok ? (await res.json() as ApiState) : null;
        } catch { return null; }
    }

    get wsUrl(): string {
        return this.baseUrl.replace(/^http/, 'ws') + '/telemetry';
    }
}

// ──────────────────────────────── EntityTree ──────────────────────────────

class EntityItem extends vscode.TreeItem {
    constructor(public readonly entity: ApiEntity) {
        super(
            `${entity.glyph}  ${entity.guid.slice(0, 12)}…`,
            vscode.TreeItemCollapsibleState.None
        );
        this.description = `(${entity.x}, ${entity.y})`;
        this.tooltip     = new vscode.MarkdownString(
            `**GUID:** \`${entity.guid}\`\n\n**Pos:** (${entity.x}, ${entity.y})`
        );
        this.iconPath = new vscode.ThemeIcon('account');
    }
}

class EntityTreeProvider implements vscode.TreeDataProvider<EntityItem> {
    private _onChange = new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this._onChange.event;
    private entities: ApiEntity[] = [];

    update(entities: ApiEntity[]) { this.entities = entities; this._onChange.fire(); }
    clear()                       { this.entities = [];        this._onChange.fire(); }

    getTreeItem(e: EntityItem) { return e; }
    getChildren(e?: EntityItem): EntityItem[] {
        if (e) { return []; }
        if (this.entities.length === 0) {
            const empty = new vscode.TreeItem('Engine offline');
            empty.iconPath = new vscode.ThemeIcon('warning');
            return [empty as EntityItem];
        }
        return this.entities.map(en => new EntityItem(en));
    }
}

// ──────────────────────────────── TelemetryPanel ─────────────────────────

class TelemetryPanel {
    static current?: TelemetryPanel;
    private readonly panel: vscode.WebviewPanel;

    static open(wsUrl: string) {
        if (TelemetryPanel.current) {
            TelemetryPanel.current.panel.reveal(vscode.ViewColumn.Two);
            return;
        }
        const p = vscode.window.createWebviewPanel(
            'cdeTelemetry', 'CDE Telemetry',
            vscode.ViewColumn.Two,
            { enableScripts: true, retainContextWhenHidden: true }
        );
        TelemetryPanel.current = new TelemetryPanel(p, wsUrl);
    }

    private constructor(p: vscode.WebviewPanel, wsUrl: string) {
        this.panel = p;
        this.panel.webview.html = TelemetryPanel.html(wsUrl);
        this.panel.onDidDispose(() => { TelemetryPanel.current = undefined; });
    }

    private static html(wsUrl: string): string {
        return /* html */`<!DOCTYPE html><html lang="en"><head>
<meta charset="UTF-8">
<style>
  * { box-sizing: border-box; }
  body { font-family: 'Cascadia Code', monospace; font-size: 12px;
         background: var(--vscode-editor-background);
         color: var(--vscode-editor-foreground);
         margin: 0; padding: 8px; display: flex; flex-direction: column; height: 100vh; }
  #toolbar { display: flex; gap: 8px; align-items: center; padding: 4px 0 8px;
             border-bottom: 1px solid var(--vscode-panel-border); flex-shrink: 0; }
  select, button { background: var(--vscode-button-secondaryBackground);
                   color: var(--vscode-button-secondaryForeground);
                   border: 1px solid var(--vscode-button-border, #555);
                   padding: 2px 8px; cursor: pointer; font-size: 11px; }
  button:hover { background: var(--vscode-button-secondaryHoverBackground); }
  #status { font-size: 11px; font-weight: bold; }
  .on  { color: #4ec9b0; } .off { color: #f44747; }
  #log { flex: 1; overflow-y: auto; padding-top: 6px; }
  .ev  { padding: 2px 4px; border-left: 3px solid #444; margin-bottom: 2px; }
  .ev.TickCompleted { border-color: #264f78; opacity: .6; }
  .ev.EntitySpawned { border-color: #4ec9b0; }
  .ev.EntityMoved   { border-color: #dcdcaa; }
  .ev.ErrorIsolated { border-color: #f44747; background: rgba(244,71,71,.08); }
  .ts   { color: var(--vscode-descriptionForeground); margin-right: 5px; font-size: 10px; }
  .kind { font-weight: bold; margin-right: 5px; }
  #cnt  { margin-left: auto; font-size: 11px; color: var(--vscode-descriptionForeground); }
</style>
</head><body>
<div id="toolbar">
  <span id="status" class="off">⬤ Offline</span>
  <select id="filter">
    <option value="">All events</option>
    <option value="TickCompleted">TickCompleted</option>
    <option value="EntitySpawned">EntitySpawned</option>
    <option value="EntityMoved">EntityMoved</option>
    <option value="ErrorIsolated">ErrorIsolated</option>
  </select>
  <label style="font-size:11px"><input type="checkbox" id="scroll" checked> Auto-scroll</label>
  <button onclick="clearLog()">Clear</button>
  <span id="cnt">0 events</span>
</div>
<div id="log"></div>
<script>
  const WS = "${wsUrl}";
  let count = 0, filter = '';
  const logEl = document.getElementById('log');
  const cntEl = document.getElementById('cnt');
  const stEl  = document.getElementById('status');
  const scEl  = document.getElementById('scroll');
  document.getElementById('filter').onchange = e => filter = e.target.value;

  function connect() {
    const ws = new WebSocket(WS);
    ws.onopen  = () => { stEl.textContent='⬤ Connected';    stEl.className='on';  };
    ws.onclose = () => { stEl.textContent='⬤ Reconnecting'; stEl.className='off';
                         setTimeout(connect, 2000); };
    ws.onerror = () => ws.close();
    ws.onmessage = ({ data }) => {
      try { push(JSON.parse(data)); } catch {}
    };
  }

  function push(ev) {
    if (filter && ev.kind !== filter) return;
    count++;
    cntEl.textContent = count + ' events';
    const el = document.createElement('div');
    el.className = 'ev ' + (ev.kind || '');
    const ts = new Date().toLocaleTimeString('en',{hour12:false});
    el.innerHTML = '<span class="ts">' + ts + '</span>'
                 + '<span class="kind">' + esc(ev.kind||'?') + '</span>'
                 + esc(JSON.stringify(ev.data||{}));
    // Ротация TickCompleted — держим не больше 200
    if (ev.kind === 'TickCompleted') {
      const old = logEl.querySelectorAll('.TickCompleted');
      if (old.length > 200) old[0].remove();
    }
    logEl.appendChild(el);
    if (scEl.checked) el.scrollIntoView({block:'end'});
  }

  function clearLog() { logEl.innerHTML=''; count=0; cntEl.textContent='0 events'; }
  function esc(s) { return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
  connect();
</script>
</body></html>`;
    }
}

// ──────────────────────────────── activate ────────────────────────────────

export function activate(context: vscode.ExtensionContext): void {
    const cfg    = () => vscode.workspace.getConfiguration('cde');
    const client = () => new EngineClient(cfg().get<string>('engineUrl', 'http://127.0.0.1:8080'));

    // Status bar
    const bar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    bar.command = 'cde.openTelemetry';
    bar.text    = '$(game) CDE: offline';
    bar.tooltip = 'Click to open Telemetry panel';
    bar.show();
    context.subscriptions.push(bar);

    // Entity tree
    const tree = new EntityTreeProvider();
    context.subscriptions.push(
        vscode.window.registerTreeDataProvider('cde.entities', tree)
    );

    // Polling loop
    let connected = false;
    let timer: ReturnType<typeof setInterval> | undefined;

    function startPolling() {
        if (timer) { return; }
        timer = setInterval(async () => {
            const state = await client().fetchState();
            if (state) {
                if (!connected) {
                    connected = true;
                    vscode.window.setStatusBarMessage('$(check) CDE: engine connected', 3000);
                }
                bar.text    = `$(game) CDE  tick#${state.tick}  |  ${state.entity_count} entities`;
                bar.tooltip = `Connected to ${cfg().get('engineUrl')}`;
                bar.color   = undefined;
                tree.update(state.entities);
            } else {
                if (connected) {
                    connected = false;
                    vscode.window.setStatusBarMessage('$(warning) CDE: engine offline', 3000);
                }
                bar.text  = '$(game) CDE: offline';
                bar.color = new vscode.ThemeColor('statusBarItem.warningForeground');
                tree.clear();
            }
        }, 1000);
    }

    function stopPolling() {
        if (timer) { clearInterval(timer); timer = undefined; }
        connected = false;
        bar.text  = '$(game) CDE: offline';
        bar.color = undefined;
        tree.clear();
    }

    // Запускаем сразу при активации
    startPolling();

    // Commands
    context.subscriptions.push(
        vscode.commands.registerCommand('cde.connect', () => {
            startPolling();
            vscode.window.showInformationMessage(
                `CDE: connecting to ${cfg().get('engineUrl')}…`
            );
        }),
        vscode.commands.registerCommand('cde.disconnect', () => {
            stopPolling();
            vscode.window.showInformationMessage('CDE: disconnected');
        }),
        vscode.commands.registerCommand('cde.refreshEntities', async () => {
            const state = await client().fetchState();
            if (state) { tree.update(state.entities); }
            else { vscode.window.showWarningMessage('CDE: engine not reachable'); }
        }),
        vscode.commands.registerCommand('cde.openTelemetry', () => {
            TelemetryPanel.open(client().wsUrl);
        }),
        vscode.commands.registerCommand('cde.spawnTestEntity', () => {
            // Будет реализован когда добавим POST /api/command
            vscode.window.showInformationMessage(
                'CDE: spawn endpoint coming in next step (Depot Data integration)'
            );
        }),
        { dispose: stopPolling }
    );
}

export function deactivate(): void {}