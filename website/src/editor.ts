import { EditorState } from '@codemirror/state';
import { EditorView, lineNumbers, highlightActiveLine, keymap, placeholder } from '@codemirror/view';
import { json } from '@codemirror/lang-json';
import { oneDark } from '@codemirror/theme-one-dark';
import { defaultKeymap } from '@codemirror/commands';

export interface Editor {
  getValue(): string;
  setValue(content: string): void;
  onChange(callback: (content: string) => void): void;
  focus(): void;
  destroy(): void;
}

export function createEditor(container: HTMLElement, initialContent: string = '', onRender?: () => void): Editor {
  let changeCallback: ((content: string) => void) | null = null;

  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged && changeCallback) {
      changeCallback(update.state.doc.toString());
    }
  });

  const state = EditorState.create({
    doc: initialContent,
    extensions: [
      lineNumbers(),
      highlightActiveLine(),
      EditorView.lineWrapping,
      json(),
      oneDark,
      keymap.of([{ key: 'Mod-Enter', run: () => { onRender?.(); return Boolean(onRender); } }, ...defaultKeymap]),
      EditorView.contentAttributes.of({ 'aria-labelledby': 'editor-label', 'aria-describedby': 'editor-hint', spellcheck: 'false' }),
      placeholder('{ type: "sprite", name: "dot", size: [1,1], palette: { x: "#ff0000" }, regions: { x: { points: [[0,0]] } } }'),
      updateListener,
    ],
  });

  const view = new EditorView({
    state,
    parent: container,
  });

  return {
    getValue(): string {
      return view.state.doc.toString();
    },

    setValue(content: string): void {
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: content,
        },
      });
    },

    onChange(callback: (content: string) => void): void {
      changeCallback = callback;
    },

    focus(): void {
      view.focus();
    },

    destroy(): void {
      view.destroy();
    },
  };
}
