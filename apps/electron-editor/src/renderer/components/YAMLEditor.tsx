import React, { useEffect, useRef } from 'react';
import Editor from '@monaco-editor/react';
import { useScenarioStore } from '../store/scenarioStore';
import { useEditorStore } from '../store/editorStore';
import YAML from 'js-yaml';
import './YAMLEditor.css';

interface YAMLEditorProps {
  onSave?: () => void;
}

const YAMLEditor: React.FC<YAMLEditorProps> = ({ onSave }) => {
  const editorRef = useRef<any>(null);
  const { scenario, setScenario } = useScenarioStore();
  const { setDirty, saveSnapshot } = useEditorStore();

  const handleEditorChange = (value: string | undefined) => {
    if (!value) return;

    try {
      const parsed = YAML.load(value) as any;
      setScenario(parsed);
      setDirty(true);
    } catch (e) {
      // YAML parse error - but we still show the error
      console.error('YAML parse error:', e);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      handleSave();
    }

    if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
      e.preventDefault();
      // Undo would be handled by editor
    }
  };

  const handleSave = () => {
    saveSnapshot('Save scenario');
    onSave?.();
  };

  const yamlContent = YAML.dump(scenario, {
    indent: 2,
    lineWidth: -1,
  });

  return (
    <div className="yaml-editor-container" onKeyDown={handleKeyDown}>
      <Editor
        defaultLanguage="yaml"
        value={yamlContent}
        onChange={handleEditorChange}
        theme="vs-dark"
        options={{
          minimap: { enabled: true, side: 'right' },
          fontSize: 13,
          fontFamily: "'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace",
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
          wordWrap: 'off',
          tabSize: 2,
          insertSpaces: true,
          formatOnPaste: true,
          formatOnType: true,
          autoClosingBrackets: 'always',
          autoClosingQuotes: 'always',
        }}
        onMount={(editor) => {
          editorRef.current = editor;
        }}
      />

      {/* Save button */}
      <div className="yaml-editor-actions">
        <button onClick={handleSave} className="save-button">
          💾 Save (Cmd+S)
        </button>
      </div>
    </div>
  );
};

export default YAMLEditor;
