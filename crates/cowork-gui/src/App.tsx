import React, { useEffect, useRef, useState, useMemo, useCallback } from 'react';
import { Layout, Menu, Button, Empty, App as AntApp, Tag, Spin } from 'antd';
import {
	FolderOutlined,
	FileTextOutlined,
	CodeOutlined,
	EyeOutlined,
	PlayCircleOutlined,
	ReloadOutlined,
	MessageOutlined,
	AppstoreOutlined,
	DatabaseOutlined,
	BranchesOutlined,
	CheckCircleOutlined,
	RocketOutlined,
	BookOutlined,
	SettingOutlined
} from '@ant-design/icons';

import { useProjectStore, useAgentStore, useUIStore } from './stores';
import { LoadingScreen, StatusBadge } from './components/common';
import { ChatPanel } from './components/chat';
import { useAppEvents, usePMAgent, useIterationActions, useChatInput } from './hooks';

import type { ChatMode, PMAction, PMAgentMessage, ChatMessage } from './stores';

import ArtifactsViewer from './components/ArtifactsViewer';
import CodeEditor from './components/CodeEditor';
import RunnerPanel from './components/RunnerPanel';
import ProjectsPanel from './components/ProjectsPanel';
import MemoryPanel from './components/MemoryPanel';
import KnowledgePanel from './components/KnowledgePanel';
import CommandPalette from './components/CommandPalette';
import IterationsPanel from './components/IterationsPanel';
import SettingsPanel from './components/SettingsPanel';

const { Sider, Content, Header, Footer } = Layout;

function App() {
	// State
	const [userInput, setUserInput] = useState('');
	const messagesContainerRef = useRef<HTMLDivElement>(null);
	const pmMessagesContainerRef = useRef<HTMLDivElement>(null);

	// Project store
	const {
		project,
		iterations,
		currentIteration,
		loading,
		loadProject,
		setCurrentIteration,
		updateCurrentIterationStatus
	} = useProjectStore();

	// Agent store
	const {
		messages,
		pmMessages,
		isProcessing,
		currentAgent,
		inputRequest,
		pmProcessing,
		setInputRequest,
		loadPMWelcomeMessage
	} = useAgentStore();

	// UI store
	const {
		activeView,
		commandPaletteVisible,
		activeArtifactTab,
		artifactsRefreshTrigger,
		codeRefreshTrigger,
		memoryRefreshTrigger,
		knowledgeRefreshTrigger,
		setActiveView,
		setCommandPaletteVisible,
		setActiveArtifactTab
	} = useUIStore();

	// Custom hooks
	useAppEvents(userInput, setUserInput);
	const { handlePMSendMessage, handlePMAction } = usePMAgent();
	const { handleSelectIteration, handleExecuteIteration, handleOpenProjectFolder, handleCommandSelect } = useIterationActions();
	const {
		inputRequest: chatInputRequest,
		handleSendUserMessage,
		handleSelectOption,
		handleSubmitFeedback,
		handleToggleThinking,
		handleCancelFeedback
	} = useChatInput();

	// Compute chat mode
	const chatMode = useMemo<ChatMode>(() => {
		if (!currentIteration) return 'disabled';
		if (currentIteration.status === 'Completed') return 'pm_agent';
		if (isProcessing || currentIteration.status === 'Running') return 'pipeline';
		return 'pipeline';
	}, [currentIteration, isProcessing]);

	// Load PM welcome message when entering PM agent mode
	useEffect(() => {
		if (chatMode === 'pm_agent' && currentIteration) {
			const pmMessages = useAgentStore.getState().pmMessages;
			if (pmMessages.length === 0) {
				loadPMWelcomeMessage(currentIteration.id);
			}
		}
	}, [chatMode, currentIteration?.id, loadPMWelcomeMessage]);

	// Auto-scroll
	useEffect(() => {
		if (messagesContainerRef.current) {
			messagesContainerRef.current.scrollTop = messagesContainerRef.current.scrollHeight;
		}
	}, [messages]);

	useEffect(() => {
		if (pmMessagesContainerRef.current && pmMessages.length > 0) {
			pmMessagesContainerRef.current.scrollTop = pmMessagesContainerRef.current.scrollHeight;
		}
	}, [pmMessages]);

	// Wrapper functions for ChatPanel callbacks
	const handleSend = useCallback(() => {
		if (chatMode === 'pm_agent') {
			handlePMSendMessage(userInput, setUserInput);
		} else {
			handleSendUserMessage(userInput, setUserInput);
		}
	}, [chatMode, userInput, handlePMSendMessage, handleSendUserMessage]);

	const handleSelectOptionWrapper = useCallback((option: Parameters<typeof handleSelectOption>[0]) => {
		handleSelectOption(option, userInput, setUserInput);
	}, [handleSelectOption, userInput]);

	const handleSubmitFeedbackWrapper = useCallback(() => {
		handleSubmitFeedback(userInput, setUserInput, updateCurrentIterationStatus);
	}, [handleSubmitFeedback, userInput, updateCurrentIterationStatus]);

	const handlePMActionWrapper = useCallback((action: PMAction) => {
		handlePMAction(action, pmMessages as (ChatMessage & { type: 'user' | 'pm_agent' })[]);
	}, [handlePMAction, pmMessages]);

	// Render content based on active view
	const renderContent = () => (
		<div style={{ height: '100%' }}>
			<div style={{ height: '100%', display: activeView === 'iterations' ? 'block' : 'none' }}>
				<IterationsPanel
					key="iterations"
					onSelectIteration={handleSelectIteration}
					selectedIterationId={currentIteration?.id}
				/>
			</div>

			<div style={{ height: '100%', display: activeView === 'projects' ? 'block' : 'none' }}>
				<ProjectsPanel key="projects" />
			</div>

			<div style={{ height: '100%', display: activeView === 'artifacts' ? 'block' : 'none' }}>
				{currentIteration ? (
					<ArtifactsViewer
						key={`artifacts-${currentIteration.id}`}
						iterationId={currentIteration.id}
						activeTab={activeArtifactTab}
						onTabChange={setActiveArtifactTab}
						refreshTrigger={artifactsRefreshTrigger}
					/>
				) : (
					<Empty description="Select an iteration" style={{ marginTop: '40px' }} />
				)}
			</div>

			<div style={{ height: '100%', display: activeView === 'code' ? 'block' : 'none' }}>
				{currentIteration ? (
					<CodeEditor
						key={`code-${currentIteration.id}`}
						iterationId={currentIteration.id}
						refreshTrigger={codeRefreshTrigger}
					/>
				) : (
					<Empty description="Select an iteration" style={{ marginTop: '40px' }} />
				)}
			</div>

			<div style={{ height: '100%', display: activeView === 'run' ? 'block' : 'none' }}>
				{currentIteration ? (
					<RunnerPanel key={`run-${currentIteration.id}`} iterationId={currentIteration.id} />
				) : (
					<Empty description="Select an iteration" style={{ marginTop: '40px' }} />
				)}
			</div>

			<div style={{ height: '100%', display: activeView === 'execution-memory' ? 'block' : 'none' }}>
				<MemoryPanel
					key={`memory-${memoryRefreshTrigger}`}
					currentSession={currentIteration?.id}
					refreshTrigger={memoryRefreshTrigger}
				/>
			</div>

			<div style={{ height: '100%', display: activeView === 'project-knowledge' ? 'block' : 'none' }}>
				<KnowledgePanel
					key={`knowledge-${knowledgeRefreshTrigger}`}
					currentSession={project?.id}
					currentIterationId={currentIteration?.id}
					refreshTrigger={knowledgeRefreshTrigger}
				/>
			</div>

			<div style={{ height: '100%', display: activeView === 'settings' ? 'block' : 'none', overflow: 'auto' }}>
				<SettingsPanel />
			</div>

			<div style={{ height: '100%', display: activeView === 'chat' ? 'block' : 'none' }}>
				{currentIteration ? (
					<ChatPanel
						messages={messages}
						pmMessages={pmMessages as (ChatMessage & { type: 'user' | 'pm_agent' })[]}
						mode={chatMode}
						isProcessing={isProcessing}
						pmProcessing={pmProcessing}
						currentAgent={currentAgent}
						iterationTitle={currentIteration.title}
						iterationDescription={currentIteration.description}
						currentStage={currentIteration.current_stage}
						inputRequest={inputRequest}
						userInput={userInput}
						messagesContainerRef={messagesContainerRef as React.RefObject<HTMLDivElement>}
						pmMessagesContainerRef={pmMessagesContainerRef as React.RefObject<HTMLDivElement>}
						onUserInputChange={setUserInput}
						onSend={handleSend}
						onSelectOption={handleSelectOptionWrapper}
						onSubmitFeedback={handleSubmitFeedbackWrapper}
						onCancelFeedback={handleCancelFeedback}
						onToggleThinking={handleToggleThinking}
						onActionClick={handlePMActionWrapper}
					/>
				) : (
					<Empty description="Select an iteration to view chat" style={{ marginTop: '40px' }} />
				)}
			</div>
		</div>
	);

	if (loading) {
		return <LoadingScreen />;
	}

	return (
		<Layout style={{ minHeight: '100vh' }}>
			<Header
				style={{
					background: '#fff',
					borderBottom: '1px solid #e8e8e8',
					padding: '0 24px',
					display: 'flex',
					alignItems: 'center',
					justifyContent: 'space-between'
				}}
			>
				<div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
					<h1 style={{ margin: 0, fontSize: '18px' }}>
						<RocketOutlined style={{ marginRight: '8px', color: '#1890ff' }} />
						Cowork Forge
					</h1>
					{project && (
						<Tag color="blue" style={{ cursor: 'pointer' }} onClick={handleOpenProjectFolder}>
							{project.name}
						</Tag>
					)}
				</div>

				<div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
					{currentIteration && (
						<>
							<StatusBadge status={currentIteration.status} />
							{(currentIteration.status === 'Draft' || currentIteration.status === 'Paused') && (
								<Button
									type="primary"
									icon={
										currentIteration.status === 'Draft' ? (
											<PlayCircleOutlined />
										) : (
											<ReloadOutlined />
										)
									}
									onClick={handleExecuteIteration}
									loading={isProcessing}
								>
									{currentIteration.status === 'Draft' ? 'Start Iteration' : 'Continue'}
								</Button>
							)}
						</>
					)}
				</div>
			</Header>

			<Layout style={{ height: 'calc(100vh - 64px - 48px)' }}>
				<Sider width={200} style={{ background: '#fff', borderRight: '1px solid #e8e8e8' }}>
					<Menu
						mode="inline"
						selectedKeys={[activeView]}
						onClick={({ key }) => setActiveView(key as typeof activeView)}
						style={{ height: '100%', borderRight: 0 }}
						items={[
							{ key: 'projects', icon: <AppstoreOutlined />, label: 'Projects' },
							{ key: 'iterations', icon: <BranchesOutlined />, label: 'Iterations' },
							{ key: 'chat', icon: <MessageOutlined />, label: 'Collaborate' },
							{ key: 'artifacts', icon: <FileTextOutlined />, label: 'Artifacts' },
							{ key: 'code', icon: <CodeOutlined />, label: 'Code' },
							{ key: 'run', icon: <PlayCircleOutlined />, label: 'Run' },
							{ key: 'execution-memory', icon: <DatabaseOutlined />, label: 'Memory' },
							{ key: 'project-knowledge', icon: <BookOutlined />, label: 'Knowledge' },
							{ type: 'divider' },
							{ key: 'settings', icon: <SettingOutlined />, label: 'Settings' }
						]}
					/>
				</Sider>

				<Content style={{ overflow: 'hidden', height: '100%', display: 'flex', flexDirection: 'column' }}>
					{renderContent()}
				</Content>
			</Layout>

			<Footer
				style={{
					background: '#fff',
					borderTop: '1px solid #e8e8e8',
					padding: '12px 24px',
					display: 'flex',
					justifyContent: 'space-between',
					alignItems: 'center'
				}}
			>
				<div style={{ fontSize: '12px', color: '#888' }}>
					{project ? (
						<>
							<span style={{ marginRight: '16px', cursor: 'pointer' }} onClick={handleOpenProjectFolder}>
								Project: <strong>{project.name}</strong>
							</span>
							<span>
								Iterations: <strong>{iterations.length}</strong>
							</span>
						</>
					) : (
						'No project loaded'
					)}
				</div>
				<div style={{ fontSize: '12px', color: '#888' }}>
					{isProcessing ? (
						<span style={{ color: '#1890ff' }}>
							<Spin size="small" style={{ marginRight: '8px' }} />
							{currentAgent ? `${currentAgent} is working...` : 'Processing...'}
						</span>
					) : (
						<span style={{ color: '#52c41a' }}>
							<CheckCircleOutlined style={{ marginRight: '4px' }} />
							Ready
						</span>
					)}
				</div>
			</Footer>

			<CommandPalette
				visible={commandPaletteVisible}
				onClose={() => setCommandPaletteVisible(false)}
				onCommandSelect={handleCommandSelect}
			/>
		</Layout>
	);
}

export default App;
