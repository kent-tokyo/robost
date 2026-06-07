import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { useSettingsStore } from './store/settingsStore';
import enTranslations from './locales/en.json';
import jaTranslations from './locales/ja.json';
import zhTranslations from './locales/zh.json';

const resources = {
  en: { translation: enTranslations },
  ja: { translation: jaTranslations },
  zh: { translation: zhTranslations },
};

// Initialize i18n
i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: useSettingsStore.getState().locale,
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
  });

// Listen for locale changes in settings store
useSettingsStore.subscribe((state, prevState) => {
  if (state.locale !== prevState.locale && state.locale) {
    i18n.changeLanguage(state.locale);
  }
});

export default i18n;
