import * as vscode from 'vscode';
import { ApiEntity } from './engineClient';

export class EntityItem extends vscode.TreeItem {
    constructor(public readonly entity: ApiEntity) {
        super(
            `${entity.glyph}  ${entity.guid.slice(0, 12)}…`,
            vscode.TreeItemCollapsibleState.None
        );
        this.description = `(${entity.x}, ${entity.y})`;
        this.tooltip = new vscode.MarkdownString(
            `**GUID:** \`${entity.guid}\`\n\n**Pos:** (${entity.x}, ${entity.y})`
        );
        this.iconPath = new vscode.ThemeIcon('account');
    }
}

export class EntityTreeProvider implements vscode.TreeDataProvider<EntityItem> {
    private _onChange = new vscode.EventEmitter<void>();
    readonly onDidChangeTreeData = this._onChange.event;
    private entities: ApiEntity[] = [];

    update(entities: ApiEntity[]) {
        this.entities = entities;
        this._onChange.fire();
    }

    clear() {
        this.entities = [];
        this._onChange.fire();
    }

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