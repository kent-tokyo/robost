import React from 'react';
import { useTranslation } from 'react-i18next';
import { ExplorerIcon, SearchIcon, RunIcon, ExtensionsIcon, HistoryIcon, SettingsIcon } from './Icons';

type Panel = 'explorer' | 'search' | 'run' | 'extensions' | 'settings' | 'history' | null;

interface ActivityBarProps {
  activePanel: Panel;
  onPanelChange: (panel: Panel) => void;
}

const ActivityBar: React.FC<ActivityBarProps> = ({ activePanel, onPanelChange }) => {
  const { t } = useTranslation();

  const panels = [
    { id: 'explorer', label: t('activityBar.explorer'), icon: <ExplorerIcon size={20} /> },
    { id: 'search', label: t('activityBar.search'), icon: <SearchIcon size={20} /> },
    { id: 'run', label: t('activityBar.run'), icon: <RunIcon size={20} /> },
    { id: 'extensions', label: t('activityBar.extensions'), icon: <ExtensionsIcon size={20} /> },
    { id: 'history', label: 'Execution History', icon: <HistoryIcon size={20} /> },
  ];

  return (
    <div className="activity-bar">
      <div>
        {panels.map((panel) => (
          <div
            key={panel.id}
            className={`activity-bar-button ${activePanel === panel.id ? 'active' : ''}`}
            onClick={() => onPanelChange(activePanel === panel.id ? null : (panel.id as Panel))}
            title={panel.label}
          >
            {panel.icon}
          </div>
        ))}
      </div>

      <div>
        <div
          className={`activity-bar-button ${activePanel === 'settings' ? 'active' : ''}`}
          onClick={() => onPanelChange(activePanel === 'settings' ? null : 'settings')}
          title={t('activityBar.settings')}
        >
          <SettingsIcon size={20} />
        </div>
      </div>
    </div>
  );
};

export default ActivityBar;
