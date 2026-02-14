import React from 'react';
import ReactDOM from 'react-dom/client';
import { ConfigProvider, theme } from 'antd';
import App from './App.jsx';
import './styles.css';

// 清新浅色主题配置
const lightTheme = {
  algorithm: theme.defaultAlgorithm,
  token: {
    // 主色调 - 清新的蓝色
    colorPrimary: '#2563eb',
    colorPrimaryHover: '#1d4ed8',
    colorPrimaryActive: '#1e40af',
    
    // 背景色 - 纯净白色系
    colorBgBase: '#ffffff',
    colorBgContainer: '#f8fafc',
    colorBgElevated: '#f1f5f9',
    colorBgLayout: '#f8fafc',
    
    // 边框颜色
    colorBorder: '#e2e8f0',
    colorBorderSecondary: '#f1f5f9',
    
    // 文字颜色
    colorText: '#1e293b',
    colorTextSecondary: '#64748b',
    colorTextTertiary: '#94a3b8',
    
    // 圆角
    borderRadius: 8,
    borderRadiusLG: 12,
    borderRadiusSM: 6,
    
    // 阴影
    boxShadow: '0 1px 3px rgba(0, 0, 0, 0.05)',
    boxShadowSecondary: '0 4px 12px rgba(0, 0, 0, 0.08)',
    
    // 字体
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
    
    // 间距
    padding: 16,
    paddingLG: 24,
    paddingSM: 12,
  },
  components: {
    Layout: {
      headerBg: '#ffffff',
      siderBg: '#f8fafc',
      bodyBg: '#ffffff',
    },
    Menu: {
      itemBg: 'transparent',
      itemSelectedBg: '#dbeafe',
      itemColor: '#64748b',
      itemSelectedColor: '#2563eb',
      itemHoverBg: '#f1f5f9',
      itemHoverColor: '#1e293b',
    },
    Button: {
      borderRadius: 8,
      boxShadow: '0 1px 2px rgba(0, 0, 0, 0.05)',
    },
    Input: {
      borderRadius: 10,
      activeShadow: '0 0 0 3px rgba(37, 99, 235, 0.1)',
    },
    Card: {
      borderRadius: 12,
      boxShadow: '0 1px 3px rgba(0, 0, 0, 0.05)',
    },
    Tabs: {
      cardBg: '#f8fafc',
      itemColor: '#64748b',
      itemSelectedColor: '#2563eb',
    },
    Tag: {
      borderRadius: 6,
    },
    Alert: {
      borderRadius: 10,
    },
    Modal: {
      borderRadius: 16,
    },
    Dropdown: {
      borderRadius: 12,
    },
  },
};

ReactDOM.createRoot(document.getElementById('root')).render(
  <ConfigProvider theme={lightTheme}>
    <App>
      <App />
    </App>
  </ConfigProvider>
);
