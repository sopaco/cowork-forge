import React from 'react';
import ReactDOM from 'react-dom/client';
import { ConfigProvider, theme } from 'antd';
import App from './App.jsx';
import './styles.css';

const darkTheme = {
  algorithm: theme.darkAlgorithm,
  token: {
    colorPrimary: '#1890ff',
    colorBgBase: '#141414',
    colorBgContainer: '#1f1f1f',
    colorBorder: '#303030',
    colorText: '#ffffff',
    colorTextSecondary: '#888888',
  },
};

ReactDOM.createRoot(document.getElementById('root')).render(
  <ConfigProvider theme={darkTheme}>
    <App />
  </ConfigProvider>
);