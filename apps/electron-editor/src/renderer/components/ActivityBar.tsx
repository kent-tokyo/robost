import React from 'react';
import { useTranslation } from 'react-i18next';

type Panel = 'explorer' | 'search' | 'run' | 'extensions' | 'settings' | 'history' | null;

interface ActivityBarProps {
  activePanel: Panel;
  onPanelChange: (panel: Panel) => void;
}

const ActivityBar: React.FC<ActivityBarProps> = ({ activePanel, onPanelChange }) => {
  const { t } = useTranslation();

  const panels = [
    { id: 'explorer', label: t('activityBar.explorer'), icon: '📁' },
    { id: 'search', label: t('activityBar.search'), icon: '🔍' },
    { id: 'run', label: t('activityBar.run'), icon: '▶️' },
    { id: 'extensions', label: t('activityBar.extensions'), icon: '🧩' },
    { id: 'history', label: 'Execution History', icon: '📊' },
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
          ⚙️
        </div>
      </div>
    </div>
  );
};

export default ActivityBar;
