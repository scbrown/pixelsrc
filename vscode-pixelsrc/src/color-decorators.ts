import * as vscode from 'vscode';

const HEX_COLOR_REGEX = /#([0-9a-fA-F]{3,8})\b/g;

export class ColorDecoratorProvider implements vscode.Disposable {
    private decorationTypes = new Map<string, vscode.TextEditorDecorationType>();
    private disposables: vscode.Disposable[] = [];

    constructor() {
        // Decorate active editor on creation
        if (vscode.window.activeTextEditor?.document.languageId === 'pixelsrc') {
            this.updateDecorations(vscode.window.activeTextEditor);
        }

        // Update decorations when active editor changes
        this.disposables.push(
            vscode.window.onDidChangeActiveTextEditor(editor => {
                if (editor?.document.languageId === 'pixelsrc') {
                    this.updateDecorations(editor);
                }
            })
        );

        // Update decorations when document changes
        this.disposables.push(
            vscode.workspace.onDidChangeTextDocument(e => {
                const editor = vscode.window.activeTextEditor;
                if (editor && e.document === editor.document &&
                    e.document.languageId === 'pixelsrc') {
                    this.updateDecorations(editor);
                }
            })
        );
    }

    private updateDecorations(editor: vscode.TextEditor): void {
        const text = editor.document.getText();
        const colorRanges = new Map<string, vscode.Range[]>();

        let match: RegExpExecArray | null;
        const regex = new RegExp(HEX_COLOR_REGEX.source, 'g');

        while ((match = regex.exec(text)) !== null) {
            const color = normalizeHexColor(match[0]);
            if (!color) {
                continue;
            }

            const startPos = editor.document.positionAt(match.index);
            const endPos = editor.document.positionAt(match.index + match[0].length);
            const range = new vscode.Range(startPos, endPos);

            const ranges = colorRanges.get(color) || [];
            ranges.push(range);
            colorRanges.set(color, ranges);
        }

        // Clear old decorations that are no longer in the document
        for (const [color, decorationType] of this.decorationTypes) {
            if (!colorRanges.has(color)) {
                editor.setDecorations(decorationType, []);
            }
        }

        // Apply decorations for each color
        for (const [color, ranges] of colorRanges) {
            let decorationType = this.decorationTypes.get(color);
            if (!decorationType) {
                decorationType = vscode.window.createTextEditorDecorationType({
                    before: {
                        contentText: ' ',
                        width: '0.8em',
                        height: '0.8em',
                        margin: '0 0.2em 0 0',
                        border: '1px solid rgba(128, 128, 128, 0.5)',
                        backgroundColor: color,
                    },
                });
                this.decorationTypes.set(color, decorationType);
            }
            editor.setDecorations(decorationType, ranges);
        }
    }

    dispose(): void {
        for (const decorationType of this.decorationTypes.values()) {
            decorationType.dispose();
        }
        this.decorationTypes.clear();

        for (const d of this.disposables) {
            d.dispose();
        }
        this.disposables = [];
    }
}

function normalizeHexColor(hex: string): string | null {
    // Remove # prefix
    const raw = hex.replace(/^#/, '');

    switch (raw.length) {
        case 3:
            // #RGB -> #RRGGBB
            return `#${raw[0]}${raw[0]}${raw[1]}${raw[1]}${raw[2]}${raw[2]}`;
        case 4:
            // #RGBA -> #RRGGBBAA
            return `#${raw[0]}${raw[0]}${raw[1]}${raw[1]}${raw[2]}${raw[2]}${raw[3]}${raw[3]}`;
        case 6:
            return `#${raw}`;
        case 8:
            return `#${raw}`;
        default:
            return null;
    }
}
