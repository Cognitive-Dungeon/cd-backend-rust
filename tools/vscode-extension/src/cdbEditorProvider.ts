import * as path from 'path';
import * as fs from 'fs';
import { posix } from 'path';
import * as vscode from 'vscode';
import { getNonce } from './util';
import { EngineClient } from './engineClient';

export class CdbEditorProvider implements vscode.CustomTextEditorProvider {
    public static readonly viewType = 'cde.data';

    public static register(context: vscode.ExtensionContext, getClient: () => EngineClient): vscode.Disposable {
        vscode.commands.registerCommand('cde.newDataFile', async () => {
            const workspaceFolders = vscode.workspace.workspaceFolders;
            if (!workspaceFolders) {
                vscode.window.showErrorMessage("Workspace required to create .cdb files");
                return;
            }
            let defFile = vscode.workspace.getConfiguration('cde').get('defaults.newFileName') + "";
            const result = await vscode.window.showInputBox({ value: defFile, placeHolder: 'Filename (e.g. game.cdb)' });

            if(result) {
                const folderUri = workspaceFolders[0].uri;
                const fileUri = folderUri.with({ path: posix.join(folderUri.path, result) });
                const writeData = Buffer.from('{ "sheets": []}', 'utf8');
                vscode.workspace.fs.writeFile(fileUri, writeData).then(() => {
                    vscode.commands.executeCommand('vscode.openWith', fileUri, CdbEditorProvider.viewType);
                });
            }
        });

        return vscode.window.registerCustomEditorProvider(
            CdbEditorProvider.viewType,
            new CdbEditorProvider(context, getClient),
            { webviewOptions: { retainContextWhenHidden: true } }
        );
    }

    constructor(
        private readonly context: vscode.ExtensionContext,
        private readonly getClient: () => EngineClient
    ) { }

    public async resolveCustomTextEditor(
        document: vscode.TextDocument,
        webviewPanel: vscode.WebviewPanel,
        _token: vscode.CancellationToken
    ): Promise<void> {
        webviewPanel.webview.options = { enableScripts: true };
        webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview, document);

        webviewPanel.webview.onDidReceiveMessage(async e => {
            switch (e.type) {
                case 'init-view':
                    webviewPanel.webview.postMessage({ type: 'init', text: document.getText(), jsonType: 'depot' });
                    return;
                case 'update':
                    // 1. Сохраняем физический файл
                    await this.updateTextDocument(document, e.data);
                    // 2. Отправляем Hot-Reload в движок
                    this.getClient().hotReload(e.data).then(ok => {
                        if(ok) vscode.window.setStatusBarMessage('$(sync) CDE: Hot-Reloaded!', 2000);
                    });
                    return;
                case 'spawnEntity':
                    // Команда на спавн от Svelte
                    this.getClient().spawnEntity(e.guid).then(ok => {
                        if(ok) vscode.window.showInformationMessage(`Spawned entity: ${e.guid}`);
                        else vscode.window.showErrorMessage(`Engine offline. Failed to spawn.`);
                    });
                    return;
                case 'pickFile':
                    vscode.window.showOpenDialog({canSelectMany: false, openLabel: 'Select'}).then(fileUri => {
                        if (fileUri && fileUri[0]) {
                            webviewPanel.webview.postMessage({
                                type: 'filePicked',
                                filePath: path.relative(document.uri.path, fileUri[0].path),
                                fileKey: e.fileKey
                            });
                        }
                    });
                    return;
            }
        });
    }

    private updateTextDocument(document: vscode.TextDocument, json: any) {
        const edit = new vscode.WorkspaceEdit();
        edit.replace(document.uri, new vscode.Range(0, 0, document.lineCount, 0), JSON.stringify(json, null, 4));
        return vscode.workspace.applyEdit(edit);
    }

    private getHtmlForWebview(webview: vscode.Webview, document: vscode.TextDocument): string {
        const scriptUri = webview.asWebviewUri(vscode.Uri.file(path.join(this.context.extensionPath, 'out', 'compiled/bundle.js')));
        const styleUri = webview.asWebviewUri(vscode.Uri.file(path.join(this.context.extensionPath, 'out', 'compiled/bundle.css')));
        
        let iconsExtensionPath = path.join(this.context.extensionPath, 'icons');
        let iconNames = fs.existsSync(iconsExtensionPath) ? fs.readdirSync(iconsExtensionPath) : [];
        const icons: any = {};
        iconNames.forEach(iconPath => {
            let filename = iconPath.split(".")[0];
            let diskPath = vscode.Uri.file(path.join(iconsExtensionPath, iconPath));
            icons[filename] = webview.asWebviewUri(diskPath);
        });
        
        const nonce = getNonce();
        const openWithSchemaEditingOn = vscode.workspace.getConfiguration('cde').get('openWithSchemaEditingOn', true);

        return /* html */`<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset='utf-8'>
            <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${webview.cspSource}; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
            <meta name='viewport' content='width=device-width,initial-scale=1'>
            <title>CDB Data Editor</title>
            <base href="${webview.asWebviewUri(document.uri)}/">
            <link rel='stylesheet' href="${styleUri}">
            <script defer nonce="${nonce}" src="${scriptUri}"></script>
        </head>
        <body>
        <script nonce="${nonce}">
            const nonce = "${nonce}";
            const icons = ${JSON.stringify(icons)};
            const openWithSchemaEditingOn = ${openWithSchemaEditingOn};
            const vscode = acquireVsCodeApi();
        </script>
        </body>
        </html>`;
    }
}