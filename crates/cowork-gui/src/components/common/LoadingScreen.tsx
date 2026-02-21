import React from 'react';
import { Spin } from 'antd';

export const LoadingScreen: React.FC = () => (
  <div
    style={{
      height: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    }}
  >
    <Spin size="large" tip="Loading...">
      <div style={{ padding: 50 }} />
    </Spin>
  </div>
);
