import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activateLspClient(context: vscode.ExtensionContext): Promise<void> {
    const config = vscode.workspace.getConfiguration('pixelsrc');
    const pxlPath = config.get<string>('lsp.path', 'pxl');

    const serverOptions: ServerOptions = {
        command: pxlPath,
        args: ['lsp'],
        transport: TransportKind.stdio,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'pixelsrc' },
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.pxl'),
        },
        outputChannelName: 'Pixelsrc LSP',
    };

    client = new LanguageClient(
        'pixelsrc',
        'Pixelsrc Language Server',
        serverOptions,
        clientOptions
    );

    context.subscriptions.push(client);
    await client.start();
}

export async function deactivateLspClient(): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
}
