import React from 'react';
import ReactDOM from 'react-dom/client';
import { enableMapSet } from 'immer';
import App, { ErrorBoundary } from './App';
import './globals.css';
import './i18n';

enableMapSet();

// Capture unhandled promise rejections and safely stringify reasons
window.addEventListener('unhandledrejection', (event) => {
  const reason = event.reason;
  const msg = reason instanceof Error
    ? reason.stack
    : String(reason?.message ?? reason);
  console.error('[Unhandled Promise Rejection]', msg);
});

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);
root.render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
);
