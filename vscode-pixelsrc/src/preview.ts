import * as vscode from 'vscode';
import { execFile } from 'child_process';
import { promisify } from 'util';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

const execFileAsync = promisify(execFile);
const readFileAsync = promisify(fs.readFile);
const unlinkAsync = promisify(fs.unlink);

export class PreviewManager implements vscode.Disposable {
    private panels = new Map<string, vscode.WebviewPanel>();
    private disposables: vscode.Disposable[] = [];

    constructor(private context: vscode.ExtensionContext) {
        // Update preview when document changes
        this.disposables.push(
            vscode.workspace.onDidChangeTextDocument(e => {
                if (e.document.languageId === 'pixelsrc') {
                    this.updatePreview(e.document);
                }
            })
        );

        // Update preview when document is saved
        this.disposables.push(
            vscode.workspace.onDidSaveTextDocument(doc => {
                if (doc.languageId === 'pixelsrc') {
                    this.updatePreview(doc);
                }
            })
        );

        // Clean up panel tracking when editor closes
        this.disposables.push(
            vscode.workspace.onDidCloseTextDocument(doc => {
                const key = doc.uri.toString();
                const panel = this.panels.get(key);
                if (panel) {
                    panel.dispose();
                    this.panels.delete(key);
                }
            })
        );
    }

    showPreview(document: vscode.TextDocument, column: vscode.ViewColumn): void {
        const key = document.uri.toString();
        const existing = this.panels.get(key);

        if (existing) {
            existing.reveal(column);
            return;
        }

        const fileName = path.basename(document.fileName, '.pxl');
        const panel = vscode.window.createWebviewPanel(
            'pixelsrc.preview',
            `Preview: ${fileName}`,
            column,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [
                    vscode.Uri.file(path.dirname(document.fileName)),
                ],
            }
        );

        panel.onDidDispose(() => {
            this.panels.delete(key);
        });

        this.panels.set(key, panel);
        this.updatePreview(document);
    }

    private debounceTimers = new Map<string, ReturnType<typeof setTimeout>>();

    private async updatePreview(document: vscode.TextDocument): Promise<void> {
        const key = document.uri.toString();
        const panel = this.panels.get(key);
        if (!panel) {
            return;
        }

        // Debounce rapid changes (300ms)
        const existingTimer = this.debounceTimers.get(key);
        if (existingTimer) {
            clearTimeout(existingTimer);
        }

        this.debounceTimers.set(key, setTimeout(async () => {
            this.debounceTimers.delete(key);
            await this.renderAndUpdate(document, panel);
        }, 300));
    }

    private async renderAndUpdate(
        document: vscode.TextDocument,
        panel: vscode.WebviewPanel
    ): Promise<void> {
        const config = vscode.workspace.getConfiguration('pixelsrc');
        const pxlPath = config.get<string>('lsp.path', 'pxl');
        const scale = config.get<number>('preview.scale', 8);
        const background = config.get<string>('preview.background', 'checkerboard');

        // Render to a temp file, then read as base64
        const tmpFile = path.join(os.tmpdir(), `pixelsrc-preview-${Date.now()}.png`);

        try {
            await execFileAsync(
                pxlPath,
                ['render', document.fileName, '-o', tmpFile, '--scale', String(scale)],
                { timeout: 10000 }
            );

            const pngBuffer = await readFileAsync(tmpFile);
            const base64Data = pngBuffer.toString('base64');
            panel.webview.html = getPreviewHtml(base64Data, background, document.fileName);
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : String(err);
            panel.webview.html = getErrorHtml(msg, document.fileName);
        } finally {
            // Clean up temp file
            try { await unlinkAsync(tmpFile); } catch { /* ignore */ }
        }
    }

    dispose(): void {
        for (const timer of this.debounceTimers.values()) {
            clearTimeout(timer);
        }
        this.debounceTimers.clear();

        for (const panel of this.panels.values()) {
            panel.dispose();
        }
        this.panels.clear();

        for (const d of this.disposables) {
            d.dispose();
        }
        this.disposables = [];
    }
}

function getPreviewHtml(base64Png: string, background: string, fileName: string): string {
    const bgStyle = getBackgroundStyle(background);
    const name = path.basename(fileName, '.pxl');

    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pixelsrc Preview: ${escapeHtml(name)}</title>
    <style>
        body {
            margin: 0;
            padding: 16px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            font-family: var(--vscode-font-family);
        }
        .preview-container {
            ${bgStyle}
            padding: 16px;
            border-radius: 4px;
            display: inline-block;
        }
        .preview-container img {
            display: block;
            image-rendering: pixelated;
            image-rendering: crisp-edges;
        }
        .file-name {
            margin-top: 12px;
            font-size: 12px;
            opacity: 0.7;
        }
    </style>
</head>
<body>
    <div class="preview-container">
        <img src="data:image/png;base64,${base64Png}" alt="${escapeHtml(name)}" />
    </div>
    <div class="file-name">${escapeHtml(fileName)}</div>
</body>
</html>`;
}

function getErrorHtml(errorMessage: string, fileName: string): string {
    const name = path.basename(fileName, '.pxl');

    return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pixelsrc Preview: ${escapeHtml(name)}</title>
    <style>
        body {
            margin: 0;
            padding: 16px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            background: var(--vscode-editor-background);
            color: var(--vscode-editor-foreground);
            font-family: var(--vscode-font-family);
        }
        .error {
            padding: 16px;
            border: 1px solid var(--vscode-errorForeground);
            border-radius: 4px;
            color: var(--vscode-errorForeground);
            max-width: 600px;
            word-break: break-word;
        }
        .error-title {
            font-weight: bold;
            margin-bottom: 8px;
        }
        pre {
            margin: 8px 0 0;
            white-space: pre-wrap;
            font-size: 12px;
            opacity: 0.8;
        }
    </style>
</head>
<body>
    <div class="error">
        <div class="error-title">Render Error</div>
        <pre>${escapeHtml(errorMessage)}</pre>
    </div>
</body>
</html>`;
}

function getBackgroundStyle(background: string): string {
    switch (background) {
        case 'checkerboard':
            return `background-image: linear-gradient(45deg, #808080 25%, transparent 25%),
                linear-gradient(-45deg, #808080 25%, transparent 25%),
                linear-gradient(45deg, transparent 75%, #808080 75%),
                linear-gradient(-45deg, transparent 75%, #808080 75%);
            background-size: 16px 16px;
            background-position: 0 0, 0 8px, 8px -8px, -8px 0;
            background-color: #a0a0a0;`;
        case 'dark':
            return 'background-color: #1e1e1e;';
        case 'light':
            return 'background-color: #ffffff;';
        case 'transparent':
        default:
            return 'background: transparent;';
    }
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}
