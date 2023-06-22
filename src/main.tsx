import React from 'react';
import ReactDOM from 'react-dom/client';
import { Global, css } from '@emotion/react';
import App from './App';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Global
      styles={css`
        html,
        body,
        #root {
          margin: 0;
          padding: 0;
          width: 100vw;
          height: 100vh;
          overflow: hidden;
        }

        :root {
          font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
          font-size: 16px;
          line-height: 24px;
          font-weight: 400;

          color: #0f0f0f;
          background-color: #f6f6f6;

          font-synthesis: none;
          text-rendering: optimizeLegibility;
          -webkit-font-smoothing: antialiased;
          -moz-osx-font-smoothing: grayscale;
          -webkit-text-size-adjust: 100%;
        }

        @media (prefers-color-scheme: dark) {
          :root {
            color: #f6f6f6;
            background-color: #2f2f2f;
          }

          a:hover {
            color: #24c8db;
          }
        }
      `}
    />
    <App />
  </React.StrictMode>,
);
