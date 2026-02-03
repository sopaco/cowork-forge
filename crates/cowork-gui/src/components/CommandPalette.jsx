// Command Palette - Quick access to all features
import { useState, useEffect, useMemo } from 'react';
import { Modal, Input, List, Typography, Tag, Space } from 'antd';
import { SearchOutlined, ThunderboltOutlined, FolderOpenOutlined, FileTextOutlined, CodeOutlined, SettingOutlined } from '@ant-design/icons';

const { Text } = Typography;

const CommandPalette = ({ visible, onClose, onCommandSelect }) => {
  const [searchText, setSearchText] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);

  // Command definitions
  const commands = useMemo(() => [
    // Project commands
    {
      id: 'create-project',
      label: 'Create New Project',
      icon: <ThunderboltOutlined />,
      category: 'Project',
      shortcut: 'Ctrl+N',
      action: () => onCommandSelect('create-project'),
    },
    {
      id: 'open-project',
      label: 'Open Project',
      icon: <FolderOpenOutlined />,
      category: 'Project',
      shortcut: 'Ctrl+O',
      action: () => onCommandSelect('open-project'),
    },
    {
      id: 'recent-projects',
      label: 'Recent Projects',
      icon: <FolderOpenOutlined />,
      category: 'Project',
      shortcut: 'Ctrl+R',
      action: () => onCommandSelect('recent-projects'),
    },
    // File commands
    {
      id: 'new-file',
      label: 'New File',
      icon: <FileTextOutlined />,
      category: 'File',
      shortcut: 'Ctrl+Shift+N',
      action: () => onCommandSelect('new-file'),
    },
    {
      id: 'save-file',
      label: 'Save File',
      icon: <FileTextOutlined />,
      category: 'File',
      shortcut: 'Ctrl+S',
      action: () => onCommandSelect('save-file'),
    },
    {
      id: 'format-code',
      label: 'Format Code',
      icon: <CodeOutlined />,
      category: 'File',
      shortcut: 'Ctrl+Shift+F',
      action: () => onCommandSelect('format-code'),
    },
    // View commands
    {
      id: 'view-chat',
      label: 'View Chat',
      icon: <FileTextOutlined />,
      category: 'View',
      shortcut: 'Ctrl+1',
      action: () => onCommandSelect('view-chat'),
    },
    {
      id: 'view-artifacts',
      label: 'View Artifacts',
      icon: <FileTextOutlined />,
      category: 'View',
      shortcut: 'Ctrl+2',
      action: () => onCommandSelect('view-artifacts'),
    },
    {
      id: 'view-code',
      label: 'View Code',
      icon: <CodeOutlined />,
      category: 'View',
      shortcut: 'Ctrl+3',
      action: () => onCommandSelect('view-code'),
    },
    {
      id: 'view-preview',
      label: 'View Preview',
      icon: <FileTextOutlined />,
      category: 'View',
      shortcut: 'Ctrl+4',
      action: () => onCommandSelect('view-preview'),
    },
    {
      id: 'view-run',
      label: 'View Run',
      icon: <ThunderboltOutlined />,
      category: 'View',
      shortcut: 'Ctrl+5',
      action: () => onCommandSelect('view-run'),
    },
    {
      id: 'view-memory',
      label: 'View Memory',
      icon: <FileTextOutlined />,
      category: 'View',
      shortcut: 'Ctrl+6',
      action: () => onCommandSelect('view-memory'),
    },
    // Tools commands
    {
      id: 'format-all',
      label: 'Format All Files',
      icon: <CodeOutlined />,
      category: 'Tools',
      shortcut: '',
      action: () => onCommandSelect('format-all'),
    },
    {
      id: 'check-formatter',
      label: 'Check Formatter Availability',
      icon: <CodeOutlined />,
      category: 'Tools',
      shortcut: '',
      action: () => onCommandSelect('check-formatter'),
    },
    {
      id: 'get-templates',
      label: 'Get Templates',
      icon: <FolderOpenOutlined />,
      category: 'Tools',
      shortcut: '',
      action: () => onCommandSelect('get-templates'),
    },
    // Settings commands
    {
      id: 'settings',
      label: 'Settings',
      icon: <SettingOutlined />,
      category: 'Settings',
      shortcut: 'Ctrl+,',
      action: () => onCommandSelect('settings'),
    },
    {
      id: 'preferences',
      label: 'Preferences',
      icon: <SettingOutlined />,
      category: 'Settings',
      shortcut: '',
      action: () => onCommandSelect('preferences'),
    },
  ], [onCommandSelect]);

  // Filter commands based on search text
  const filteredCommands = useMemo(() => {
    if (!searchText) {
      return commands;
    }
    
    const searchLower = searchText.toLowerCase();
    return commands.filter(cmd => 
      cmd.label.toLowerCase().includes(searchLower) ||
      cmd.category.toLowerCase().includes(searchLower)
    );
  }, [commands, searchText]);

  // Group commands by category
  const groupedCommands = useMemo(() => {
    const groups = {};
    filteredCommands.forEach(cmd => {
      if (!groups[cmd.category]) {
        groups[cmd.category] = [];
      }
      groups[cmd.category].push(cmd);
    });
    return groups;
  }, [filteredCommands]);

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e) => {
      if (!visible) return;
      
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault();
          setSelectedIndex(prev => 
            Math.min(prev + 1, filteredCommands.length - 1)
          );
          break;
        case 'ArrowUp':
          e.preventDefault();
          setSelectedIndex(prev => Math.max(prev - 1, 0));
          break;
        case 'Enter':
          e.preventDefault();
          if (filteredCommands[selectedIndex]) {
            filteredCommands[selectedIndex].action();
            handleClose();
          }
          break;
        case 'Escape':
          e.preventDefault();
          handleClose();
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [visible, filteredCommands, selectedIndex]);

  // Reset search and selection when modal opens
  useEffect(() => {
    if (visible) {
      setSearchText('');
      setSelectedIndex(0);
    }
  }, [visible]);

  const handleClose = () => {
    onClose();
  };

  const handleSearchChange = (e) => {
    setSearchText(e.target.value);
    setSelectedIndex(0);
  };

  const flattenedCommands = useMemo(() => {
    return Object.values(groupedCommands).flat();
  }, [groupedCommands]);

  return (
    <Modal
      title="Command Palette"
      open={visible}
      onCancel={handleClose}
      footer={null}
      width={600}
      centered
    >
      <Input
        placeholder="Search commands..."
        prefix={<SearchOutlined />}
        value={searchText}
        onChange={handleSearchChange}
        autoFocus
        style={{ marginBottom: 16 }}
        size="large"
      />
      
      <List
        style={{ maxHeight: 400, overflow: 'auto' }}
        dataSource={flattenedCommands}
        renderItem={(item, index) => (
          <List.Item
            key={item.id}
            onClick={() => {
              item.action();
              handleClose();
            }}
            style={{
              cursor: 'pointer',
              padding: '12px',
              backgroundColor: index === selectedIndex ? '#1890ff22' : 'transparent',
              borderRadius: '4px',
            }}
          >
            <List.Item.Meta
              avatar={item.icon}
              title={
                <Space>
                  <Text>{item.label}</Text>
                  <Tag color="blue">{item.category}</Tag>
                </Space>
              }
              description={item.shortcut && (
                <Text type="secondary" style={{ fontSize: '12px' }}>
                  {item.shortcut}
                </Text>
              )}
            />
          </List.Item>
        )}
      />
    </Modal>
  );
};

export default CommandPalette;