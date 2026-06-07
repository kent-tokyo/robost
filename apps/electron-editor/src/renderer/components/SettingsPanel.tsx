import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSettingsStore } from '../store/settingsStore';
import './SettingsPanel.css';

const SettingsPanel: React.FC = () => {
  const { t } = useTranslation();
  const {
    theme,
    locale,
    autoSave,
    autoSaveInterval,
    apiKeyOpenAI,
    apiKeyAnthropic,
    setTheme,
    setLocale,
    setAutoSave,
    setAutoSaveInterval,
    setApiKeyOpenAI,
    setApiKeyAnthropic,
  } = useSettingsStore();

  const [tempApiKeyOpenAI, setTempApiKeyOpenAI] = useState(apiKeyOpenAI);
  const [tempApiKeyAnthropic, setTempApiKeyAnthropic] = useState(apiKeyAnthropic);
  const [tempAutoSaveInterval, setTempAutoSaveInterval] = useState(autoSaveInterval);
  const [showApiKeys, setShowApiKeys] = useState(false);

  const handleSaveSettings = () => {
    setApiKeyOpenAI(tempApiKeyOpenAI);
    setApiKeyAnthropic(tempApiKeyAnthropic);
    setAutoSaveInterval(tempAutoSaveInterval);
  };

  const handleResetSettings = () => {
    setTempApiKeyOpenAI('');
    setTempApiKeyAnthropic('');
    setTempAutoSaveInterval(5000);
    setAutoSave(true);
    setTheme('dark');
    setLocale('en');
  };

  return (
    <div className="settings-panel">
      <div className="settings-content">
        {/* Appearance Section */}
        <div className="settings-section">
          <h3 className="settings-section-title">{t('settings.appearance')}</h3>

          {/* Theme Setting */}
          <div className="settings-item">
            <label>{t('settings.theme')}</label>
            <div className="settings-control">
              <button
                className={`theme-button ${theme === 'dark' ? 'active' : ''}`}
                onClick={() => setTheme('dark')}
              >
                🌙 {t('settings.darkMode')}
              </button>
              <button
                className={`theme-button ${theme === 'light' ? 'active' : ''}`}
                onClick={() => setTheme('light')}
              >
                ☀️ {t('settings.lightMode')}
              </button>
            </div>
          </div>

          {/* Language Setting */}
          <div className="settings-item">
            <label>{t('settings.language')}</label>
            <select
              value={locale}
              onChange={(e) => setLocale(e.target.value as 'en' | 'ja' | 'zh')}
              className="settings-select"
            >
              <option value="en">{t('settings.english')}</option>
              <option value="ja">{t('settings.japanese')}</option>
              <option value="zh">{t('settings.chinese')}</option>
            </select>
          </div>
        </div>

        {/* Behavior Section */}
        <div className="settings-section">
          <h3 className="settings-section-title">{t('settings.behavior')}</h3>

          {/* Auto-save Setting */}
          <div className="settings-item">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={autoSave}
                onChange={(e) => setAutoSave(e.target.checked)}
              />
              <span>{t('settings.autoSave')}</span>
            </label>
          </div>

          {/* Auto-save Interval */}
          {autoSave && (
            <div className="settings-item">
              <label>{t('settings.autoSaveInterval')}</label>
              <input
                type="number"
                value={tempAutoSaveInterval}
                onChange={(e) => setTempAutoSaveInterval(parseInt(e.target.value) || 5000)}
                className="settings-input"
                min="1000"
                step="1000"
              />
            </div>
          )}
        </div>

        {/* API Keys Section */}
        <div className="settings-section">
          <h3 className="settings-section-title">{t('settings.apiKeys')}</h3>

          <button
            className="toggle-api-keys"
            onClick={() => setShowApiKeys(!showApiKeys)}
          >
            {showApiKeys ? '▼' : '▶'} {t('settings.apiKeys')}
          </button>

          {showApiKeys && (
            <div className="api-keys-container">
              {/* OpenAI API Key */}
              <div className="settings-item">
                <label>{t('settings.openaiApiKey')}</label>
                <input
                  type="password"
                  value={tempApiKeyOpenAI}
                  onChange={(e) => setTempApiKeyOpenAI(e.target.value)}
                  className="settings-input"
                  placeholder="sk-..."
                />
              </div>

              {/* Anthropic API Key */}
              <div className="settings-item">
                <label>{t('settings.anthropicApiKey')}</label>
                <input
                  type="password"
                  value={tempApiKeyAnthropic}
                  onChange={(e) => setTempApiKeyAnthropic(e.target.value)}
                  className="settings-input"
                  placeholder="sk-ant-..."
                />
              </div>

              <div style={{ fontSize: '11px', color: 'var(--color-text-secondary)', marginTop: '8px' }}>
                Your API keys are stored locally and never shared.
              </div>
            </div>
          )}
        </div>

        {/* Action Buttons */}
        <div className="settings-actions">
          <button className="settings-button primary" onClick={handleSaveSettings}>
            {t('settings.save')}
          </button>
          <button className="settings-button secondary" onClick={handleResetSettings}>
            {t('common.reset')}
          </button>
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
