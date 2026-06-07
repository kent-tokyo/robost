import React from 'react';
import { useTranslation } from 'react-i18next';

const StatusBar: React.FC = () => {
  const { t } = useTranslation();

  return (
    <div className="status-bar">
      <div className="status-bar-left">
        <div className="status-bar-item">{t('statusBar.utf8')}</div>
        <div className="status-bar-item">{t('statusBar.lf')}</div>
        <div className="status-bar-item">{t('statusBar.yaml')}</div>
      </div>
      <div className="status-bar-right">
        <div className="status-bar-item">Ln 1, Col 1</div>
      </div>
    </div>
  );
};

export default StatusBar;
