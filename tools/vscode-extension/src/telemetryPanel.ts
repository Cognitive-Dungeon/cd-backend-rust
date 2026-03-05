import * as vscode from 'vscode';

export class TelemetryPanel {
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
  body { font-family: 'Cascadia Code', monospace; font-size: 12px; background: var(--vscode-editor-background); color: var(--vscode-editor-foreground); margin: 0; padding: 8px; display: flex; flex-direction: column; height: 100vh; }
  #toolbar { display: flex; gap: 8px; align-items: center; padding: 4px 0 8px; border-bottom: 1px solid var(--vscode-panel-border); flex-shrink: 0; }
  select, button { background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground); border: 1px solid var(--vscode-button-border, #555); padding: 2px 8px; cursor: pointer; font-size: 11px; }
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
  </select>
  <label style="font-size:11px"><input type="checkbox" id="scroll" checked> Auto-scroll</label>
  <button onclick="clearLog()">Clear</button>
  <span id="cnt">0 events</span>
</div>
<div id="log"></div>
<script>
  const WS = "${wsUrl}";
  let count = 0, filter = '';
  const logEl = document.getElementById('log'), cntEl = document.getElementById('cnt'), stEl  = document.getElementById('status'), scEl  = document.getElementById('scroll');
  document.getElementById('filter').onchange = e => filter = e.target.value;

  function connect() {
    const ws = new WebSocket(WS);
    ws.onopen  = () => { stEl.textContent='⬤ Connected'; stEl.className='on'; };
    ws.onclose = () => { stEl.textContent='⬤ Reconnecting'; stEl.className='off'; setTimeout(connect, 2000); };
    ws.onerror = () => ws.close();
    ws.onmessage = ({ data }) => { try { push(JSON.parse(data)); } catch {} };
  }

  function push(ev) {
    if (filter && ev.kind !== filter) return;
    count++; cntEl.textContent = count + ' events';
    const el = document.createElement('div'); el.className = 'ev ' + (ev.kind || '');
    const ts = new Date().toLocaleTimeString('en',{hour12:false});
    el.innerHTML = '<span class="ts">' + ts + '</span><span class="kind">' + esc(ev.kind||'?') + '</span>' + esc(JSON.stringify(ev.data||{}));
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