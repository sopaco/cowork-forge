import React from 'react';
import { Badge } from 'antd';

interface StatusBadgeProps {
  status?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ status }) => {
  const statusLower = status?.toLowerCase();
  switch (statusLower) {
    case 'completed':
      return <Badge status="success" text="Completed" />;
    case 'running':
      return <Badge status="processing" text="Running" />;
    case 'paused':
      return <Badge status="warning" text="Paused" />;
    case 'failed':
      return <Badge status="error" text="Failed" />;
    default:
      return <Badge status="default" text="Draft" />;
  }
};
