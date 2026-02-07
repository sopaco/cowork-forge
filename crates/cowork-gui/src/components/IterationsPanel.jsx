import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  Card,
  Button,
  Modal,
  Input,
  Tag,
  Empty,
  message,
  Spin,
  Space,
  Tooltip,
  Progress,
  Badge,
  Timeline,
  Radio,
  Form,
  Select,
} from 'antd';
import {
  PlusOutlined,
  PlayCircleOutlined,
  PauseCircleOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  DeleteOutlined,
  ReloadOutlined,
  RedoOutlined,
  BranchesOutlined,
  RocketOutlined,
  CodeOutlined,
  FileTextOutlined,
  EyeOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';

const { TextArea } = Input;
const { Option } = Select;

// Stage definitions
const STAGES = [
  { key: 'idea', label: 'Idea', icon: <RocketOutlined />, color: '#1890ff' },
  { key: 'prd', label: 'PRD', icon: <FileTextOutlined />, color: '#52c41a' },
  { key: 'design', label: 'Design', icon: <EyeOutlined />, color: '#722ed1' },
  { key: 'plan', label: 'Plan', icon: <FileTextOutlined />, color: '#fa8c16' },
  { key: 'coding', label: 'Coding', icon: <CodeOutlined />, color: '#13c2c2' },
  { key: 'check', label: 'Check', icon: <CheckCircleOutlined />, color: '#eb2f96' },
  { key: 'delivery', label: 'Delivery', icon: <RocketOutlined />, color: '#52c41a' },
];

const IterationsPanel = ({ onSelectIteration, selectedIterationId, onExecuteStatusChange }) => {
  const [iterations, setIterations] = useState([]);
  const [project, setProject] = useState(null);
  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showDetailsModal, setShowDetailsModal] = useState(false);
  const [showInitProjectModal, setShowInitProjectModal] = useState(false);
  const [selectedIteration, setSelectedIteration] = useState(null);
  const [executingId, setExecutingId] = useState(null);
  const [newProjectName, setNewProjectName] = useState('');

  // Form state
  const [newIterationTitle, setNewIterationTitle] = useState('');
  const [newIterationDescription, setNewIterationDescription] = useState('');
  const [newIterationBase, setNewIterationBase] = useState(null);
  const [newIterationInheritance, setNewIterationInheritance] = useState('full');

  const loadData = async () => {
    setLoading(true);
    try {
      // Load project
      const projectData = await invoke('gui_get_project');
      setProject(projectData);

      // Load iterations
      const iterationsData = await invoke('gui_get_iterations');
      setIterations(iterationsData || []);
    } catch (error) {
      console.error('[IterationsPanel] Failed to load data:', error);
      message.error('Failed to load data: ' + error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();

    // Listen for events
        const unlistenCreated = listen('iteration_created', () => loadData());
        const unlistenStarted = listen('iteration_started', () => {
          // Delay refresh to allow backend to update the iteration state
          setTimeout(() => {
            loadData();
          }, 500);
        });
        const unlistenContinued = listen('iteration_continued', () => {
          // Delay refresh to allow backend to update the iteration state
          setTimeout(() => {
            loadData();
          }, 500);
        });
        const unlistenAgentEvent = listen('agent_event', (event) => {
          const content = event.payload?.content || '';
          // Refresh data when a stage starts (to get updated current_stage)
          if (content.includes('Starting stage:')) {
            setTimeout(() => {
              loadData();
            }, 100);
          }
        });
        const unlistenCompleted = listen('iteration_completed', () => {
          loadData();
          setExecutingId(null);
        });
        const unlistenFailed = listen('iteration_failed', () => {
          loadData();
          setExecutingId(null);
        });
    
        return () => {
          unlistenCreated.then(fn => fn()).catch(() => {});
          unlistenStarted.then(fn => fn()).catch(() => {});
          unlistenContinued.then(fn => fn()).catch(() => {});
          unlistenAgentEvent.then(fn => fn()).catch(() => {});
          unlistenCompleted.then(fn => fn()).catch(() => {});
          unlistenFailed.then(fn => fn()).catch(() => {});
        };  }, []);

  const handleCreateIteration = async () => {
    if (!newIterationTitle.trim()) {
      message.warning('Please enter a title');
      return;
    }

    try {
      const request = {
        title: newIterationTitle,
        description: newIterationDescription || newIterationTitle,
        base_iteration_id: newIterationBase,
        inheritance: newIterationInheritance,
      };

      await invoke('gui_create_iteration', { request });
      message.success('Iteration created successfully');
      setShowCreateModal(false);
      resetForm();
      loadData();
    } catch (error) {
      message.error('Failed to create iteration: ' + error);
    }
  };

  const handleExecuteIteration = async (iterationId) => {
    try {
      setExecutingId(iterationId);
      // Notify parent about execution status change
      onExecuteStatusChange?.(iterationId, 'running');
      await invoke('gui_execute_iteration', { iterationId });
      message.info('Iteration execution started');
    } catch (error) {
      message.error('Failed to execute iteration: ' + error);
      setExecutingId(null);
      onExecuteStatusChange?.(iterationId, 'error');
    }
  };

  const handleContinueIteration = async (iterationId) => {
    try {
      setExecutingId(iterationId);
      // Notify parent about execution status change
      onExecuteStatusChange?.(iterationId, 'running');
      // Auto-select the iteration being continued
      onSelectIteration?.(iterationId);
      await invoke('gui_continue_iteration', { iterationId });
      message.info('Iteration continued');
    } catch (error) {
      message.error('Failed to continue iteration: ' + error);
      setExecutingId(null);
      onExecuteStatusChange?.(iterationId, 'error');
    }
  };

  const handleRetryIteration = async (iterationId) => {
    try {
      setExecutingId(iterationId);
      // Notify parent about execution status change
      onExecuteStatusChange?.(iterationId, 'running');
      // Auto-select the iteration being retried
      onSelectIteration?.(iterationId);
      await invoke('gui_retry_iteration', { iterationId });
      message.info('Retrying iteration...');
    } catch (error) {
      message.error('Failed to retry iteration: ' + error);
      setExecutingId(null);
      onExecuteStatusChange?.(iterationId, 'error');
    }
  };

  const handleDeleteIteration = (iteration) => {
    Modal.confirm({
      title: 'Delete Iteration',
      content: `Are you sure you want to delete "${iteration.title}"?`,
      okText: 'Delete',
      okType: 'danger',
      onOk: async () => {
        try {
          await invoke('gui_delete_iteration', { iterationId: iteration.id });
          message.success('Iteration deleted');
          loadData();
        } catch (error) {
          message.error('Failed to delete iteration: ' + error);
        }
      },
    });
  };

  const handleViewDetails = (iteration) => {
    setSelectedIteration(iteration);
    setShowDetailsModal(true);
  };

  const resetForm = () => {
    setNewIterationTitle('');
    setNewIterationDescription('');
    setNewIterationBase(null);
    setNewIterationInheritance('full');
  };

  const handleInitProject = async () => {
    if (!newProjectName.trim()) {
      message.warning('Please enter a project name');
      return;
    }

    try {
      await invoke('gui_init_project', { name: newProjectName });
      message.success('Project initialized successfully');
      setShowInitProjectModal(false);
      setNewProjectName('');
      loadData();
    } catch (error) {
      message.error('Failed to initialize project: ' + error);
    }
  };

  const getStatusColor = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed': return 'success';
      case 'running': return 'processing';
      case 'paused': return 'warning';
      case 'failed': return 'error';
      case 'draft': return 'default';
      default: return 'default';
    }
  };

  const getStatusIcon = (status) => {
    switch (status?.toLowerCase()) {
      case 'completed': return <CheckCircleOutlined style={{ color: '#52c41a' }} />;
      case 'running': return <PlayCircleOutlined style={{ color: '#1890ff' }} />;
      case 'paused': return <PauseCircleOutlined style={{ color: '#faad14' }} />;
      case 'failed': return <CloseCircleOutlined style={{ color: '#ff4d4f' }} />;
      default: return null;
    }
  };

  const calculateProgress = (completedStages) => {
    if (!completedStages) return 0;
    return Math.round((completedStages.length / STAGES.length) * 100);
  };

  const formatDate = (dateString) => {
    if (!dateString) return 'N/A';
    return new Date(dateString).toLocaleString();
  };

  if (loading && iterations.length === 0) {
    return (
      <div style={{ padding: '40px', textAlign: 'center' }}>
        <Spin size="large" />
        <div style={{ marginTop: '16px', color: '#999' }}>Loading iterations...</div>
      </div>
    );
  }

  if (!project) {
    return (
      <div style={{ padding: '40px' }}>
        <Empty
          description="No project initialized"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        >
          <Button type="primary" onClick={() => setShowInitProjectModal(true)}>
            Initialize Project
          </Button>
        </Empty>

        {/* Initialize Project Modal */}
        <Modal
          title="Initialize Project"
          open={showInitProjectModal}
          onOk={handleInitProject}
          onCancel={() => {
            setShowInitProjectModal(false);
            setNewProjectName('');
          }}
          okText="Initialize"
          cancelText="Cancel"
        >
          <div style={{ marginBottom: '16px' }}>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold' }}>
              Project Name:
            </label>
            <Input
              value={newProjectName}
              onChange={(e) => setNewProjectName(e.target.value)}
              placeholder="Enter project name"
              autoFocus
              onPressEnter={handleInitProject}
            />
          </div>
        </Modal>
      </div>
    );
  }

  return (
    <div style={{ padding: '24px', height: '100%', overflow: 'auto' }}>
      {/* Header */}
      <div style={{ marginBottom: '24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2 style={{ margin: 0 }}>{project.name}</h2>
          <div style={{ color: '#888', fontSize: '14px', marginTop: '4px' }}>
            {iterations.length} iteration{iterations.length !== 1 ? 's' : ''}
          </div>
        </div>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setShowCreateModal(true)}
        >
          New Iteration
        </Button>
      </div>

      {/* Iterations List */}
      {iterations.length === 0 ? (
        <Empty
          description="No iterations yet"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        >
          <Button type="primary" onClick={() => setShowCreateModal(true)}>
            Create First Iteration
          </Button>
        </Empty>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {iterations.map((iteration) => (
            <Card
              key={iteration.id}
              hoverable
              className={selectedIterationId === iteration.id ? 'selected-iteration' : ''}
              style={{
                border: selectedIterationId === iteration.id ? '2px solid #1890ff' : undefined,
              }}
              onClick={() => onSelectIteration?.(iteration.id)}
              actions={[
                iteration.status === 'Draft' && (
                  <Button
                    type="link"
                    icon={<PlayCircleOutlined />}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleExecuteIteration(iteration.id);
                    }}
                  >
                    Start
                  </Button>
                ),
                iteration.status === 'Paused' && (
                  <Button
                    type="link"
                    icon={<ReloadOutlined />}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleContinueIteration(iteration.id);
                    }}
                  >
                    Continue
                  </Button>
                ),
                iteration.status === 'Failed' && (
                  <Button
                    type="link"
                    icon={<RedoOutlined />}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRetryIteration(iteration.id);
                    }}
                  >
                    Retry
                  </Button>
                ),
                iteration.status === 'Running' && (
                  <Spin size="small" style={{ marginRight: '8px' }} />
                ),
                <Button
                  type="link"
                  icon={<EyeOutlined />}
                  onClick={(e) => {
                    e.stopPropagation();
                    handleViewDetails(iteration);
                  }}
                >
                  Details
                </Button>,
                iteration.status !== 'Running' && (
                  <Button
                    type="link"
                    danger
                    icon={<DeleteOutlined />}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteIteration(iteration);
                    }}
                  >
                    Delete
                  </Button>
                ),
              ].filter(Boolean)}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div style={{ flex: 1 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
                    <span style={{ fontSize: '18px', fontWeight: 'bold' }}>
                      #{iteration.number}
                    </span>
                    <span style={{ fontSize: '16px' }}>{iteration.title}</span>
                    <Badge
                      status={getStatusColor(iteration.status)}
                      text={
                        <span style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                          {getStatusIcon(iteration.status)}
                          {iteration.status}
                        </span>
                      }
                    />
                  </div>

                  <div style={{ color: '#666', marginBottom: '12px' }}>
                    {iteration.description}
                  </div>

                  {iteration.base_iteration_id && (
                    <div style={{ fontSize: '12px', color: '#888', marginBottom: '8px' }}>
                      <BranchesOutlined style={{ marginRight: '4px' }} />
                      Based on iteration: {iteration.base_iteration_id.substring(0, 12)}...
                      <Tag size="small" style={{ marginLeft: '8px' }}>
                        {iteration.inheritance}
                      </Tag>
                    </div>
                  )}

                  {/* Progress */}
                  <div style={{ marginTop: '12px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                      <span style={{ fontSize: '12px', color: '#888' }}>Progress</span>
                      <span style={{ fontSize: '12px', color: '#888' }}>
                        {calculateProgress(iteration.completed_stages)}%
                      </span>
                    </div>
                    <Progress
                      percent={calculateProgress(iteration.completed_stages)}
                      size="small"
                      status={iteration.status === 'Failed' ? 'exception' : 'active'}
                    />
                  </div>

                  {/* Current Stage */}
                  {iteration.current_stage && (
                    <div style={{ marginTop: '8px', fontSize: '12px', color: '#1890ff' }}>
                      Current: {iteration.current_stage}
                    </div>
                  )}

                  {/* Stages */}
                  <div style={{ marginTop: '12px', display: 'flex', flexWrap: 'wrap', gap: '4px' }}>
                    {STAGES.map((stage) => {
                      // For running/paused iterations, only show stages before current_stage as completed
                      // For completed iterations, use completed_stages
                      const currentStageIndex = STAGES.findIndex(s => s.key === iteration.current_stage);
                      const stageIndex = STAGES.findIndex(s => s.key === stage.key);

                      const isCompleted = iteration.status === 'Completed'
                        ? iteration.completed_stages?.includes(stage.key)
                        : currentStageIndex >= 0 && stageIndex < currentStageIndex;

                      const isCurrent = iteration.current_stage === stage.key;

                      return (
                        <Tag
                          key={stage.key}
                          size="small"
                          color={isCompleted ? 'success' : isCurrent ? 'processing' : 'default'}
                          style={{
                            opacity: isCompleted || isCurrent ? 1 : 0.5,
                          }}
                        >
                          {isCompleted && <CheckCircleOutlined style={{ marginRight: '4px' }} />}
                          {stage.label}
                        </Tag>
                      );
                    })}
                  </div>
                </div>

                <div style={{ textAlign: 'right', color: '#888', fontSize: '12px' }}>
                  <div>Created</div>
                  <div>{formatDate(iteration.created_at)}</div>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Create Iteration Modal */}
      <Modal
        title="Create New Iteration"
        open={showCreateModal}
        onOk={handleCreateIteration}
        onCancel={() => {
          setShowCreateModal(false);
          resetForm();
        }}
        width={600}
      >
        <Form layout="vertical">
          <Form.Item label="Title" required>
            <Input
              value={newIterationTitle}
              onChange={(e) => setNewIterationTitle(e.target.value)}
              placeholder="e.g., Add user authentication"
              autoFocus
            />
          </Form.Item>

          <Form.Item label="Description">
            <TextArea
              value={newIterationDescription}
              onChange={(e) => setNewIterationDescription(e.target.value)}
              placeholder="Describe what you want to achieve in this iteration..."
              rows={4}
            />
          </Form.Item>

          {iterations.length > 0 && (
            <>
              <Form.Item label="Base Iteration (Optional)">
                <Select
                  value={newIterationBase}
                  onChange={setNewIterationBase}
                  placeholder="Select a base iteration to inherit from"
                  allowClear
                >
                  {iterations
                    .filter((i) => i.status === 'Completed')
                    .map((iteration) => (
                      <Option key={iteration.id} value={iteration.id}>
                        #{iteration.number} - {iteration.title}
                      </Option>
                    ))}
                </Select>
                <div style={{ marginTop: '4px', fontSize: '12px', color: '#888' }}>
                  Leave empty to start from scratch (Genesis iteration)
                </div>
              </Form.Item>

              {newIterationBase && (
                <Form.Item label="Inheritance Mode">
                  <Radio.Group
                    value={newIterationInheritance}
                    onChange={(e) => setNewIterationInheritance(e.target.value)}
                  >
                    <Radio.Button value="full">Full (All code)</Radio.Button>
                    <Radio.Button value="partial">Partial (Artifacts only)</Radio.Button>
                    <Radio.Button value="none">None (Fresh start)</Radio.Button>
                  </Radio.Group>
                </Form.Item>
              )}
            </>
          )}
        </Form>
      </Modal>

      {/* Iteration Details Modal */}
      <Modal
        open={showDetailsModal}
        onCancel={() => setShowDetailsModal(false)}
        footer={null}
        width={680}
      >
        {selectedIteration && (
          <div>
            {/* Compact Header */}
            <div style={{ 
              marginBottom: '20px',
              paddingBottom: '16px',
              borderBottom: '1px solid #e8e8e8'
            }}>
              <div style={{ 
                display: 'flex', 
                alignItems: 'center', 
                gap: '10px',
                marginBottom: '8px'
              }}>
                <span style={{ 
                  fontSize: '13px', 
                  fontWeight: 600,
                  background: '#f0f0f0',
                  padding: '2px 10px',
                  borderRadius: '4px'
                }}>
                  #{selectedIteration.number}
                </span>
                <Badge 
                  status={getStatusColor(selectedIteration.status)} 
                  text={selectedIteration.status}
                />
              </div>
              <h2 style={{ 
                margin: 0, 
                fontSize: '20px', 
                fontWeight: 600,
                marginBottom: '6px'
              }}>
                {selectedIteration.title}
              </h2>
              <div style={{ 
                fontSize: '12px', 
                color: '#999',
                fontFamily: 'monospace'
              }}>
                ID: {selectedIteration.id}
              </div>
            </div>

            {/* Description */}
            <div style={{ marginBottom: '16px' }}>
              <div style={{
                padding: '12px',
                background: '#fafafa',
                borderRadius: '6px',
                lineHeight: '1.5',
                fontSize: '14px',
                color: '#666'
              }}>
                {selectedIteration.description}
              </div>
            </div>

            {/* Inheritance */}
            {selectedIteration.base_iteration_id && (
              <div style={{ marginBottom: '16px' }}>
                <div style={{ 
                  fontSize: '13px', 
                  color: '#999', 
                  marginBottom: '4px' 
                }}>
                  Based on: <code style={{ 
                    background: '#f5f5f5', 
                    padding: '2px 6px', 
                    borderRadius: '3px',
                    fontSize: '11px'
                  }}>
                    {selectedIteration.base_iteration_id.substring(0, 20)}...
                  </code>
                  <Tag style={{ marginLeft: '6px', fontSize: '11px' }}>
                    {selectedIteration.inheritance}
                  </Tag>
                </div>
              </div>
            )}

            {/* Progress & Stages */}
            <div style={{ marginBottom: '16px' }}>
              <div style={{ 
                display: 'flex', 
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: '10px'
              }}>
                <span style={{ fontSize: '13px', fontWeight: 600 }}>Progress</span>
                <span style={{ fontSize: '13px', color: '#666' }}>
                  {calculateProgress(selectedIteration.completed_stages)}%
                </span>
              </div>
              <Progress 
                percent={calculateProgress(selectedIteration.completed_stages)}
                strokeColor="#52c41a"
                size="small"
                style={{ marginBottom: '12px' }}
              />
              <div style={{ 
                display: 'flex', 
                flexWrap: 'wrap', 
                gap: '6px'
              }}>
                {STAGES.map((stage) => {
                  const currentStageIndex = STAGES.findIndex(s => s.key === selectedIteration.current_stage);
                  const stageIndex = STAGES.findIndex(s => s.key === stage.key);

                  const isCompleted = selectedIteration.status === 'Completed'
                    ? selectedIteration.completed_stages?.includes(stage.key)
                    : currentStageIndex >= 0 && stageIndex < currentStageIndex;

                  const isCurrent = selectedIteration.current_stage === stage.key;

                  return (
                    <Tag
                      key={stage.key}
                      color={isCompleted ? 'success' : isCurrent ? 'processing' : 'default'}
                      style={{
                        margin: 0,
                        fontSize: '12px',
                        borderRadius: '4px',
                        opacity: isCompleted || isCurrent ? 1 : 0.4
                      }}
                    >
                      {stage.label}
                    </Tag>
                  );
                })}
              </div>
            </div>

            {/* Timestamps */}
            <div style={{
              display: 'flex',
              gap: '24px',
              paddingTop: '16px',
              borderTop: '1px solid #e8e8e8'
            }}>
              <div>
                <div style={{ fontSize: '12px', color: '#999', marginBottom: '2px' }}>
                  Created
                </div>
                <div style={{ fontSize: '13px', color: '#333' }}>
                  {formatDate(selectedIteration.created_at)}
                </div>
              </div>
              {selectedIteration.completed_at && (
                <div>
                  <div style={{ fontSize: '12px', color: '#999', marginBottom: '2px' }}>
                    Completed
                  </div>
                  <div style={{ fontSize: '13px', color: '#333' }}>
                    {formatDate(selectedIteration.completed_at)}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};

export default IterationsPanel;
