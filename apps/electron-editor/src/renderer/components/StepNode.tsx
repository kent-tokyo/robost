import React, { useState } from 'react';
import { Handle, Position } from 'reactflow';
import './StepNode.css';

interface StepNodeProps {
  data: {
    label: string;
    type: string;
  };
  isConnecting?: boolean;
  onSelect?: () => void;
  selected?: boolean;
}

const StepNode: React.FC<StepNodeProps> = ({ data, isConnecting, onSelect, selected }) => {
  const [showMenu, setShowMenu] = useState(false);

  const getIconForType = (type: string) => {
    const icons: Record<string, string> = {
      click_image: '🖱️',
      wait_image: '⏳',
      type: '⌨️',
      press: '📌',
      script: '📝',
      if: '❓',
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

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setShowMenu(!showMenu);
  };

  return (
    <div className={`step-node ${selected ? 'selected' : ''}`} onClick={onSelect}>
      <Handle type="target" position={Position.Top} />

      <div className="step-node-header">
        <span className="step-node-icon">{getIconForType(data.type)}</span>
        <span className="step-node-label">{data.label}</span>
      </div>

      <div
        className="step-node-context"
        onContextMenu={handleContextMenu}
        title="Right-click for options"
      >
        ⋯
      </div>

      {showMenu && (
        <div className="step-node-menu">
          <div className="step-node-menu-item">Edit</div>
          <div className="step-node-menu-item">Duplicate</div>
          <div className="step-node-menu-item">Delete</div>
        </div>
      )}

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

export default StepNode;
