import * as vscode from 'vscode';
import { EngineClient } from './engineClient';
import { EntityTreeProvider } from './entityTreeProvider';
import { TelemetryPanel } from './telemetryPanel';
import { CdbEditorProvider } from './cdbEditorProvider';

export function activate(context: vscode.ExtensionContext): void {
    console.log("CDE SDK extension activating...");
    // 1. Инициализация клиента
    const cfg = () => vscode.workspace.getConfiguration('cde');
    const getClient = () => new EngineClient(cfg().get<string>('engineUrl', 'http://127.0.0.1:8080'));

    // 2. Инициализация UI компонентов
    const statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBar.command = 'cde.openTelemetry';
    statusBar.text = '$(game) CDE: offline';
    statusBar.show();
    context.subscriptions.push(statusBar);

    const treeProvider = new EntityTreeProvider();
    context.subscriptions.push(vscode.window.registerTreeDataProvider('cde.entities', treeProvider));

    // 3. Регистрация редактора данных .cdb
    context.subscriptions.push(CdbEditorProvider.register(context, getClient));

    // 4. Логика поллинга движка
    let connected = false;
    let timer: ReturnType<typeof setInterval> | undefined;

    function startPolling() {
        if (timer) return;
        timer = setInterval(async () => {
            const state = await getClient().fetchState();
            if (state) {
                if (!connected) {
                    connected = true;
                    vscode.window.setStatusBarMessage('$(check) CDE: engine connected', 3000);
                }
                statusBar.text = `$(game) CDE  tick#${state.tick}  |  ${state.entity_count} entities`;
                treeProvider.update(state.entities);
            } else {
                if (connected) {
                    connected = false;
                    vscode.window.setStatusBarMessage('$(warning) CDE: engine offline', 3000);
                }
                statusBar.text = '$(game) CDE: offline';
                treeProvider.clear();
            }
        }, 1000);
    }

    function stopPolling() {
        if (timer) { clearInterval(timer); timer = undefined; }
        connected = false;
        statusBar.text = '$(game) CDE: offline';
        treeProvider.clear();
    }

    // Запускаем поллинг при старте
    startPolling();

    // 5. Регистрация команд
    context.subscriptions.push(
        vscode.commands.registerCommand('cde.connect', () => {
            startPolling();
            vscode.window.showInformationMessage(`CDE: connecting...`);
        }),
        vscode.commands.registerCommand('cde.disconnect', () => {
            stopPolling();
            vscode.window.showInformationMessage('CDE: disconnected');
        }),
        vscode.commands.registerCommand('cde.refreshEntities', async () => {
            const state = await getClient().fetchState();
            if (state) treeProvider.update(state.entities);
        }),
        vscode.commands.registerCommand('cde.openTelemetry', () => {
            TelemetryPanel.open(getClient().wsUrl);
        }),
        { dispose: stopPolling }
    );


    // Хелпер для получения URI активной вкладки (неважно, текст это или таблица)
    const getActiveUri = (): vscode.Uri | undefined => {
        const tabInput = vscode.window.tabGroups.activeTabGroup.activeTab?.input;
        if (
            tabInput instanceof vscode.TabInputCustom || 
            tabInput instanceof vscode.TabInputText
        ) {
            return tabInput.uri;
        }
        return undefined;
    };

    // 1. Открыть как JSON (текст)
    context.subscriptions.push(vscode.commands.registerCommand('cde.openCdbAsJson', async (uri?: vscode.Uri) => {
        // Если вызвали из меню - uri передается. Если из палитры - ищем сами.
        const targetUri = uri || getActiveUri();
        if (targetUri) {
            await vscode.commands.executeCommand('vscode.openWith', targetUri, 'default');
        } else {
            vscode.window.showErrorMessage("No active .cdb file found");
        }
    }));

    // 2. Открыть как Таблицу (Custom Editor)
    context.subscriptions.push(vscode.commands.registerCommand('cde.openCdbAsTable', async (uri?: vscode.Uri) => {
        const targetUri = uri || getActiveUri();
        if (targetUri) {
            await vscode.commands.executeCommand('vscode.openWith', targetUri, 'cde.data');
        } else {
            vscode.window.showErrorMessage("No active .cdb file found");
        }
    }));
}

export function deactivate(): void { }