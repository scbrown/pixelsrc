import * as vscode from 'vscode';
import { activateLspClient, deactivateLspClient } from './lsp-client';
import { PreviewManager } from './preview';
import { ColorDecoratorProvider } from './color-decorators';

let previewManager: PreviewManager | undefined;
let colorDecorator: ColorDecoratorProvider | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
    const config = vscode.workspace.getConfiguration('pixelsrc');

    // Start LSP client
    if (config.get<boolean>('lsp.enabled', true)) {
        await activateLspClient(context);
    }

    // Initialize preview manager
    if (config.get<boolean>('preview.enabled', true)) {
        previewManager = new PreviewManager(context);
    }

    // Initialize color decorators
    if (config.get<boolean>('colorDecorators.enabled', true)) {
        colorDecorator = new ColorDecoratorProvider();
        context.subscriptions.push(colorDecorator);
    }

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('pixelsrc.openPreview', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'pixelsrc') {
                previewManager?.showPreview(editor.document, vscode.ViewColumn.Active);
            }
        }),

        vscode.commands.registerCommand('pixelsrc.openPreviewToSide', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor && editor.document.languageId === 'pixelsrc') {
                previewManager?.showPreview(editor.document, vscode.ViewColumn.Beside);
            }
        }),

        vscode.commands.registerCommand('pixelsrc.exportPng', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'pixelsrc') {
                vscode.window.showWarningMessage('Open a .pxl file to export as PNG');
                return;
            }
            await exportPng(editor.document);
        })
    );

    // Watch for configuration changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('pixelsrc.lsp.enabled') ||
                e.affectsConfiguration('pixelsrc.lsp.path')) {
                vscode.window.showInformationMessage(
                    'Pixelsrc: Reload window to apply LSP settings changes.'
                );
            }
        })
    );
}

export async function deactivate(): Promise<void> {
    await deactivateLspClient();
    previewManager?.dispose();
    colorDecorator?.dispose();
}

async function exportPng(document: vscode.TextDocument): Promise<void> {
    const config = vscode.workspace.getConfiguration('pixelsrc');
    const pxlPath = config.get<string>('lsp.path', 'pxl');

    const saveUri = await vscode.window.showSaveDialog({
        defaultUri: vscode.Uri.file(document.fileName.replace(/\.pxl$/, '.png')),
        filters: { 'PNG Images': ['png'] }
    });

    if (!saveUri) {
        return;
    }

    const { execFile } = require('child_process') as typeof import('child_process');
    const { promisify } = require('util') as typeof import('util');
    const execFileAsync = promisify(execFile);

    try {
        await execFileAsync(pxlPath, ['render', document.fileName, '-o', saveUri.fsPath]);
        vscode.window.showInformationMessage(`Exported: ${saveUri.fsPath}`);
    } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err);
        vscode.window.showErrorMessage(`Export failed: ${msg}`);
    }
}
