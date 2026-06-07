import React from 'react';
import {
  Files, Search, Play, Puzzle, History, Settings,
  FilePlus, FolderOpen, Save, SaveAll, Sparkles,
  Workflow, List, Code2, Monitor, Undo2, Redo2,
  LayoutDashboard, Group, Ungroup, Square,
  RefreshCw, AlertTriangle, Camera, Copy, Check,
  ClipboardList, Moon, Sun, Pause,
  MessageSquare, Trash2, CopyPlus,
  ChevronDown, ChevronRight, X,
  MousePointer2, Hourglass, Keyboard, Pin, FileCode,
  Diamond, Repeat, RotateCcw, RefreshCcw, Tag,
  Calculator, Terminal, BookOpen, Phone, Shield, Box, MapPin,
} from 'lucide-react';

// Navigation & UI icons
export const ExplorerIcon = (p: any) => <Files {...p} />;
export const SearchIcon = (p: any) => <Search {...p} />;
export const RunIcon = (p: any) => <Play {...p} />;
export const ExtensionsIcon = (p: any) => <Puzzle {...p} />;
export const HistoryIcon = (p: any) => <History {...p} />;
export const SettingsIcon = (p: any) => <Settings {...p} />;

// File operations
export const NewFileIcon = (p: any) => <FilePlus {...p} />;
export const OpenFolderIcon = (p: any) => <FolderOpen {...p} />;
export const SaveIcon = (p: any) => <Save {...p} />;
export const SaveAsIcon = (p: any) => <SaveAll {...p} />;

// Editor & Canvas
export const SparklesIcon = (p: any) => <Sparkles {...p} />;
export const CanvasIcon = (p: any) => <Workflow {...p} />;
export const ListIcon = (p: any) => <List {...p} />;
export const CodeIcon = (p: any) => <Code2 {...p} />;
export const ScreenIcon = (p: any) => <Monitor {...p} />;
export const UndoIcon = (p: any) => <Undo2 {...p} />;
export const RedoIcon = (p: any) => <Redo2 {...p} />;
export const LayoutIcon = (p: any) => <LayoutDashboard {...p} />;
export const GroupIcon = (p: any) => <Group {...p} />;
export const UngroupIcon = (p: any) => <Ungroup {...p} />;
export const StopIcon = (p: any) => <Square {...p} />;

// Operations & Status
export const RefreshIcon = (p: any) => <RefreshCw {...p} />;
export const AlertIcon = (p: any) => <AlertTriangle {...p} />;
export const CameraIcon = (p: any) => <Camera {...p} />;
export const CopyIcon = (p: any) => <Copy {...p} />;
export const CheckIcon = (p: any) => <Check {...p} />;
export const ClipboardListIcon = (p: any) => <ClipboardList {...p} />;
export const MoonIcon = (p: any) => <Moon {...p} />;
export const SunIcon = (p: any) => <Sun {...p} />;
export const PauseIcon = (p: any) => <Pause {...p} />;
export const CommentIcon = (p: any) => <MessageSquare {...p} />;
export const TrashIcon = (p: any) => <Trash2 {...p} />;
export const CopyPlusIcon = (p: any) => <CopyPlus {...p} />;
export const ChevronDownIcon = (p: any) => <ChevronDown {...p} />;
export const ChevronRightIcon = (p: any) => <ChevronRight {...p} />;
export const CloseIcon = (p: any) => <X {...p} />;

// Step type icons (shared mapping)
const STEP_ICON_MAP: Record<string, React.ComponentType<any>> = {
  click_image: MousePointer2,
  wait_image: Hourglass,
  type: Keyboard,
  press: Pin,
  script: FileCode,
  if: Diamond,
  foreach: Repeat,
  while: RotateCcw,
  repeat: RefreshCcw,
  set: Tag,
  calc: Calculator,
  log: ClipboardList,
  shell: Terminal,
  library: BookOpen,
  call_scenario: Phone,
  try_catch: Shield,
  group: Box,
};

export const getStepTypeIcon = (type: string, size = 12): React.ReactNode => {
  const Icon = STEP_ICON_MAP[type] ?? MapPin;
  return <Icon size={size} />;
};
