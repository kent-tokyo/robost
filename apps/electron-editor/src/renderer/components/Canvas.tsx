import React, { useCallback, useMemo, useEffect, useState } from 'react';
import ReactFlow, {
  Node,
  Edge,
  Controls,
  Background,
  MiniMap,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  Panel,
  useReactFlow,
} from 'reactflow';
import 'reactflow/dist/style.css';
import StepNode from './StepNode';
import SearchCanvas from './SearchCanvas';
import { SearchIcon, LayoutIcon, GroupIcon, UngroupIcon, RunIcon, StopIcon } from './Icons';
import { useScenarioStore } from '../store/scenarioStore';
import { useEditorStore } from '../store/editorStore';
import { useRunStore } from '../store/runStore';
import { useCanvasStore } from '../store/canvasStore';
import { useRpaServer } from '../hooks/useRpaServer';
import { autoLayoutNodes } from '../utils/canvasLayout';
import './Canvas.css';

interface CanvasProps {
  onNodeSelect?: (nodeId: string) => void;
}

const NODE_TYPES = { stepNode: StepNode } as const;

const Canvas: React.FC<CanvasProps> = ({ onNodeSelect }) => {
  const { scenario, canvasLayout, updateCanvasNodes, updateCanvasEdges, updateCanvasZoom, updateCanvasPan, addStep, deleteStep, groupSteps, ungroupSteps, pasteStep } = useScenarioStore();
  const { saveSnapshot } = useEditorStore();
  const { isRunning, currentStepIndex, totalSteps, elapsedMs } = useRunStore();
  const { runScenario, stopScenario } = useRpaServer();
  const { getZoom, getViewport, setCenter, screenToFlowPosition } = useReactFlow();
  const { getSelectedNodeIds, clearSelection, searchHighlightIds, getFromClipboard, clearClipboard, filterType } = useCanvasStore();
  const [showSearchModal, setShowSearchModal] = useState(false);

  // Initialize nodes from scenario steps
  const initialNodes: Node[] = scenario.steps.map((step, index) => ({
    id: step.id,
    type: 'stepNode',
    data: {
      label: step.name,
      type: step.type,
      comment: step.comment,
      isGrouped: step.type === 'group',
      childCount: step.childSteps?.length || 0,
    },
    position: canvasLayout.nodes[index]?.position || { x: 250, y: index * 150 },
  }));

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(canvasLayout.edges);

  const onConnect = useCallback(
    (connection: Connection) => {
      const newEdges = addEdge(connection, edges);
      setEdges(newEdges);
      updateCanvasEdges(newEdges);
    },
    [edges, setEdges, updateCanvasEdges]
  );

  const onNodesChangeWithSave = useCallback(
    (changes: any[]) => {
      onNodesChange(changes);
      setNodes((nds) => {
        updateCanvasNodes(nds);
        return nds;
      });
      if (changes.some((c) => c.type === 'position' && c.dragging === false)) {
        saveSnapshot('Reposition node');
      }
    },
    [onNodesChange, setNodes, updateCanvasNodes, saveSnapshot]
  );


  const onDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const onDrop = useCallback((e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const { x: nodeX, y: nodeY } = screenToFlowPosition({ x: e.clientX, y: e.clientY });

    // Try to get template data first (from template gallery)
    const templateData = e.dataTransfer.getData('templateData');
    if (templateData) {
      try {
        const steps = JSON.parse(templateData);
        let xOffset = 0;
        steps.forEach((step: any, index: number) => {
          const newStep = {
            ...step,
            position: {
              x: nodeX + xOffset,
              y: nodeY,
            },
          };
          addStep(newStep);
          xOffset += 250;
        });
        saveSnapshot(`Add template: ${steps[0]?.name || 'unknown'}`);
      } catch (err) {
        console.error('Failed to parse template data:', err);
      }
      return;
    }

    // Fallback to generic step type (legacy)
    const stepType = e.dataTransfer.getData('stepType');
    if (!stepType) return;

    const newStep = {
      id: `step-${Date.now()}`,
      name: stepType,
      type: stepType as any,
      data: {},
      position: {
        x: nodeX,
        y: nodeY,
      },
    };

    addStep(newStep);
    saveSnapshot(`Add step: ${stepType}`);
  }, [addStep, saveSnapshot, screenToFlowPosition]);

  const handleDeleteNode = useCallback((nodeId: string) => {
    deleteStep(nodeId);
    saveSnapshot('Delete step');
  }, [deleteStep, saveSnapshot]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      // Cmd+F / Ctrl+F: Search
      if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        setShowSearchModal(true);
      }

      // Cmd+C / Ctrl+C: Copy selected
      if ((e.metaKey || e.ctrlKey) && e.key === 'c') {
        const selectedIds = getSelectedNodeIds();
        if (selectedIds.length > 0) {
          e.preventDefault();
          // Copy first selected node
          const step = scenario.steps.find((s) => s.id === selectedIds[0]);
          if (step) {
            // Handled by StepNode component
          }
        }
      }

      // Cmd+V / Ctrl+V: Paste
      if ((e.metaKey || e.ctrlKey) && e.key === 'v') {
        const clipboard = getFromClipboard();
        if (clipboard) {
          e.preventDefault();
          pasteStep(clipboard);
          saveSnapshot('Paste step');
          clearClipboard();
        }
      }

      // Cmd+D / Ctrl+D: Duplicate selected (handled by StepNode)
      // Delete: Delete selected nodes
      if (e.key === 'Delete' || e.key === 'Backspace') {
        const selectedIds = getSelectedNodeIds();
        if (selectedIds.length > 0 && !isEditing()) {
          e.preventDefault();
          selectedIds.forEach((id) => deleteStep(id));
          saveSnapshot('Delete steps');
          clearSelection();
        }
      }
    },
    [getSelectedNodeIds, scenario.steps, getFromClipboard, pasteStep, saveSnapshot, clearClipboard, deleteStep, clearSelection]
  );

  const isEditing = useCallback(() => {
    const activeElement = document.activeElement;
    return activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA';
  }, []);

  const handleAutoLayout = useCallback(() => {
    const positions = autoLayoutNodes(scenario.steps);
    const newNodes = nodes.map((node) => ({
      ...node,
      position: positions[node.id] || node.position,
    }));
    setNodes(newNodes);
    updateCanvasNodes(newNodes);
    saveSnapshot('Auto-layout canvas');
  }, [nodes, scenario.steps, setNodes, updateCanvasNodes, saveSnapshot]);

  const handleGroupSelected = useCallback(() => {
    const selectedIds = getSelectedNodeIds();
    if (selectedIds.length < 2) {
      alert('Select at least 2 steps to group');
      return;
    }

    const groupName = prompt('Enter group name:', 'New Group');
    if (groupName) {
      groupSteps(selectedIds, groupName);
      saveSnapshot(`Group steps: ${groupName}`);
      clearSelection();
    }
  }, [getSelectedNodeIds, groupSteps, saveSnapshot, clearSelection]);

  const handleUngroupSelected = useCallback(() => {
    const selectedIds = getSelectedNodeIds();
    selectedIds.forEach((id) => {
      const step = scenario.steps.find((s) => s.id === id);
      if (step?.type === 'group') {
        ungroupSteps(id);
      }
    });
    saveSnapshot('Ungroup steps');
    clearSelection();
  }, [getSelectedNodeIds, scenario.steps, ungroupSteps, saveSnapshot, clearSelection]);

  // Sync canvas state to store
  useEffect(() => {
    const zoom = getZoom();
    const viewport = getViewport();
    updateCanvasZoom(zoom);
    updateCanvasPan(viewport.x, viewport.y);
  }, [getZoom, getViewport, updateCanvasZoom, updateCanvasPan]);

  // Keyboard shortcuts
  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  // Sync scenario.steps to canvas nodes whenever steps change
  useEffect(() => {
    setNodes((currentNodes) => {
      const positionMap = new Map(currentNodes.map((n) => [n.id, n.position]));
      return scenario.steps.map((step, index) => ({
        id: step.id,
        type: 'stepNode',
        data: {
          label: step.name,
          type: step.type,
          comment: step.comment,
          isGrouped: step.type === 'group',
          childCount: step.childSteps?.length || 0,
        },
        position: positionMap.get(step.id)
          ?? canvasLayout.nodes[index]?.position
          ?? { x: 250, y: index * 150 },
      }));
    });
  }, [scenario.steps, canvasLayout.nodes, setNodes]);

  return (
    <div className="canvas-container" onDragOver={onDragOver} onDrop={onDrop}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChangeWithSave}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={NODE_TYPES}
        fitView
      >
        <Background color="#aaa" gap={16} />
        <Controls />
        <MiniMap />

        <Panel position="top-left" className="canvas-toolbar">
          <button
            className="canvas-toolbar-btn"
            onClick={() => setShowSearchModal(true)}
            title="Search canvas (Cmd+F)"
            style={{ display: 'flex', alignItems: 'center', gap: '4px' }}
          >
            <SearchIcon size={14} /> Search
          </button>
          <button
            className="canvas-toolbar-btn"
            onClick={handleAutoLayout}
            title="Auto-layout nodes"
            style={{ display: 'flex', alignItems: 'center', gap: '4px' }}
          >
            <LayoutIcon size={14} /> Layout
          </button>
          <button
            className="canvas-toolbar-btn"
            onClick={handleGroupSelected}
            title="Group selected steps"
            disabled={getSelectedNodeIds().length < 2}
            style={{ display: 'flex', alignItems: 'center', gap: '4px' }}
          >
            <GroupIcon size={14} /> Group
          </button>
          <button
            className="canvas-toolbar-btn"
            onClick={handleUngroupSelected}
            title="Ungroup selected steps"
            style={{ display: 'flex', alignItems: 'center', gap: '4px' }}
          >
            <UngroupIcon size={14} /> Ungroup
          </button>
        </Panel>

        <Panel position="top-right" className="canvas-panel">
          <div style={{ padding: '8px', fontSize: '12px', color: '#ccc' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
              <SearchIcon size={12} /> {nodes.length} steps
            </div>
            {isRunning && (
              <div style={{ marginTop: '4px', fontSize: '11px' }}>
                Running: {currentStepIndex}/{totalSteps}
              </div>
            )}
          </div>
        </Panel>

        <Panel position="bottom-right" className="canvas-controls">
          <button
            onClick={() => (isRunning ? stopScenario() : runScenario())}
            style={{
              padding: '8px 12px',
              backgroundColor: isRunning ? '#f48771' : '#6a9955',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '12px',
              fontWeight: 'bold',
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
            }}
          >
            {isRunning ? (
              <><StopIcon size={14} /> Stop</>
            ) : (
              <><RunIcon size={14} /> Run</>
            )}
          </button>
        </Panel>
      </ReactFlow>

      <SearchCanvas isOpen={showSearchModal} onClose={() => setShowSearchModal(false)} />
    </div>
  );
};

export default Canvas;
