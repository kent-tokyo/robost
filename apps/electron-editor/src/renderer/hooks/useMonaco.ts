import { useEffect, useRef } from 'react';
import * as monaco from 'monaco-editor';

export const useMonaco = () => {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);

  useEffect(() => {
    // Register YAML language
    if (!monaco.languages.getEncodedLanguageId('yaml')) {
      monaco.languages.register({ id: 'yaml' });
    }

    // YAML syntax highlighting
    monaco.languages.setMonarchTokensProvider('yaml', {
      defaultToken: '',
      ignoreCase: true,
      tokenizer: {
        root: [
          [/^[ \t]*[a-zA-Z_][a-zA-Z0-9_-]*:/, 'key'],
          [/^[ \t]*-/, 'operator'],
          [/"[^"]*"/, 'string'],
          [/'[^']*'/, 'string'],
          [/[0-9]+/, 'number'],
          [/(true|false|null)/, 'keyword'],
          [/#.*/, 'comment'],
        ],
      },
    });

    // Define theme colors
    monaco.editor.defineTheme('robost-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'key', foreground: '9CDCFE', fontStyle: 'bold' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'keyword', foreground: '569CD6' },
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'operator', foreground: 'D4D4D4' },
      ],
      colors: {
        'editor.background': '#1e1e1e',
        'editor.foreground': '#cccccc',
        'editor.lineNumbersColumn.background': '#1e1e1e',
        'editor.lineNumber.foreground': '#858585',
        'editor.selectionBackground': '#264F78',
        'editorCursor.foreground': '#aeafad',
      },
    });

    monaco.editor.setTheme('robost-dark');

    return () => {
      // Cleanup if needed
    };
  }, []);

  return { editorRef };
};
