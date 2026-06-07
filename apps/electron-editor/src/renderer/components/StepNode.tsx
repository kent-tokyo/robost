import React, { useState, useCallback, useRef, useEffect } from 'react';
import { Handle, Position } from 'reactflow';
import { useCanvasStore } from '../store/canvasStore';
import { useScenarioStore } from '../store/scenarioStore';
import { useEditorStore } from '../store/editorStore';
import './StepNode.css';

interface StepNodeProps {
  id: string;
  data: {
    label: string;
    type: string;
    comment?: string;
    isGrouped?: boolean;
    childCount?: number;
  };
  isConnecting?: boolean;
  selected?: boolean;
}

const StepNode: React.FC<StepNodeProps> = ({ id, data, isConnecting, selected }) => {
  const [showMenu, setShowMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const { isNodeSelected, toggleNodeSelection, copyToClipboard, selectNode, clearSelection } = useCanvasStore();
  const { updateStep, deleteStep, duplicateStep } = useScenarioStore();
  const { saveSnapshot, setSelectedNodeId } = useEditorStore();

  const getIconForType = (type: string) => {
    const icons: Record<string, string> = {
      click_image: '🖱️',
      wait_image: '⏳',
      type: '⌨️',
      press: '📌',
      script: '📝',
      if: '◇',
      foreach: '🔄',
      while: '↩️',
      repeat: '🔁',
      set: '📌',
      calc: '🧮',
      log: '📋',
      shell: '💻',
      library: '📚',
      call_scenario: '📞',
      try_catch: '🛡️',
      group: '📦',
    };
    return icons[type] || '📍';
  };

  const getNodeColorClass = (type: string) => {
    const colorMap: Record<string, string> = {
      if: 'node-conditional-if',
      foreach: 'node-conditional-loop',
      while: 'node-conditional-loop',
      try_catch: 'node-conditional-try',
      group: 'node-group',
    };
    return colorMap[type] || '';
  };

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setShowMenu(!showMenu);
  }, [showMenu]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.ctrlKey || e.metaKey) {
      e.stopPropagation();
      toggleNodeSelection(id);
    } else {
      e.stopPropagation();
      if (!isNodeSelected(id)) {
        clearSelection();
        selectNode(id);
        setSelectedNodeId(id);
      }
    }
  }, [id, isNodeSelected, toggleNodeSelection, selectNode, clearSelection, setSelectedNodeId]);

  const handleCopy = useCallback(() => {
    const step = { id, name: data.label, type: data.type, data: {} };
    copyToClipboard(step as any);
    setShowMenu(false);
    saveSnapshot('Copy step');
  }, [id, data.label, data.type, copyToClipboard, saveSnapshot]);

  const handleDuplicate = useCallback(() => {
    duplicateStep(id);
    setShowMenu(false);
    saveSnapshot('Duplicate step');
  }, [id, duplicateStep, saveSnapshot]);

  const handleDelete = useCallback(() => {
    deleteStep(id);
    setShowMenu(false);
    saveSnapshot('Delete step');
  }, [id, deleteStep, saveSnapshot]);

  const handleAddComment = useCallback(() => {
    const comment = prompt('Add a comment for this step:');
    if (comment) {
      updateStep(id, { comment });
      saveSnapshot('Add comment');
    }
    setShowMenu(false);
  }, [id, updateStep, saveSnapshot]);

  // Close menu on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowMenu(false);
      }
    };

    if (showMenu) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showMenu]);

  const nodeClass = `step-node ${selected ? 'selected' : ''} ${isNodeSelected(id) ? 'multi-selected' : ''} ${getNodeColorClass(data.type)}`;

  return (
    <div className={nodeClass} onMouseDown={handleMouseDown}>
      <Handle type="target" position={Position.Top} />

      <div className="step-node-header">
        <span className="step-node-icon">{getIconForType(data.type)}</span>
        <div className="step-node-label-container">
          <span className="step-node-label">{data.label}</span>
          {data.childCount ? <span className="step-node-badge">{data.childCount}</span> : null}
        </div>
      </div>

      {data.comment && <div className="step-node-comment">💬 {data.comment}</div>}

      <div
        className="step-node-context"
        onContextMenu={handleContextMenu}
        title="Right-click for options"
      >
        ⋯
      </div>

      {showMenu && (
        <div className="step-node-menu" ref={menuRef}>
          <div className="step-node-menu-item" onClick={handleCopy}>
            📋 Copy
          </div>
          <div className="step-node-menu-item" onClick={handleDuplicate}>
            🔀 Duplicate
          </div>
          <div className="step-node-menu-item" onClick={handleAddComment}>
            💬 Add Comment
          </div>
          <div className="step-node-menu-item step-node-menu-item-danger" onClick={handleDelete}>
            🗑️ Delete
          </div>
        </div>
      )}

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

export default StepNode;
