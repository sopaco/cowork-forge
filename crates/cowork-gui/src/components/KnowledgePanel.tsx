import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { App, Card, Input, Button, Tag, Empty, Spin, Modal, Tabs, Typography, Space, Divider, Timeline, Tooltip, Badge } from "antd";
import { SearchOutlined, EyeOutlined, BookOutlined, ClockCircleOutlined, ReloadOutlined, RocketOutlined, FileTextOutlined, CodeOutlined, CheckCircleOutlined, CloseCircleOutlined, PlusOutlined, LoadingOutlined } from "@ant-design/icons";

const { Text, Paragraph, Title } = Typography;

interface Knowledge {
  iteration_id: string;
  title: string;
  idea_summary?: string;
  design_summary?: string;
  plan_summary?: string;
  code_structure?: string;
  created_at: string;
  tech_stack?: string[];
  key_decisions?: string[];
  key_patterns?: string[];
  known_issues?: string[];
}

interface KnowledgeListResult {
  knowledge_list: Knowledge[];
}

interface IterationInfo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: string;
  current_stage: string | null;
  created_at: string;
}

interface KnowledgePanelProps {
  currentSession?: string;
  currentIterationId?: string | null;
  refreshTrigger?: number;
}

const KnowledgePanel: React.FC<KnowledgePanelProps> = ({ currentSession, currentIterationId, refreshTrigger }) => {
  const { message } = App.useApp();
  const [knowledgeList, setKnowledgeList] = useState<Knowledge[]>([]);
  const [loading, setLoading] = useState(false);
  // Iteration IDs whose knowledge generation is running. This mirrors the
  // backend's in-flight set instead of being pure component state, so that a
  // tab switch (which unmounts this panel) does not lose the "generating" UI.
  const [generatingIds, setGeneratingIds] = useState<string[]>([]);
  const [selectedKnowledge, setSelectedKnowledge] = useState<Knowledge | null>(null);
  const [searchText, setSearchText] = useState("");
  const [iterations, setIterations] = useState<IterationInfo[]>([]);
  const [loadingIterations, setLoadingIterations] = useState(false);

  const loadProjectKnowledge = useCallback(async () => {
    if (!currentSession) {
      setKnowledgeList([]);
      return;
    }

    setLoading(true);
    try {
      const result = await invoke<KnowledgeListResult>("gui_get_project_knowledge", { projectId: currentSession });
      setKnowledgeList(result.knowledge_list || []);
    } catch (error) {
      console.error("[KnowledgePanel] Failed to load project knowledge:", error);
      message.error("Failed to load knowledge: " + error);
      setKnowledgeList([]);
    } finally {
      setLoading(false);
    }
  }, [currentSession, message]);

  /**
   * Re-sync the in-flight generation set from the backend.
   *
   * This is what makes the UI honest after a remount: the backend owns the
   * truth, so a panel that was unmounted mid-generation restores the running
   * state instead of offering a clickable Generate button.
   */
  const syncGenerationStatus = useCallback(async () => {
    try {
      const ids = await invoke<string[]>("gui_get_knowledge_generation_status");
      setGeneratingIds(Array.isArray(ids) ? ids : []);
    } catch (error) {
      console.error("[KnowledgePanel] Failed to read generation status:", error);
    }
  }, []);

  const loadIterations = useCallback(async () => {
    if (!currentSession) {
      setIterations([]);
      return;
    }

    setLoadingIterations(true);
    try {
      const iterationsData = await invoke<IterationInfo[]>("gui_get_iterations");
      const completedIterations = (iterationsData || []).filter(
        (iter) => iter.status === "Completed"
      );
      setIterations(completedIterations);
    } catch (error) {
      console.error("[KnowledgePanel] Failed to load iterations:", error);
      setIterations([]);
    } finally {
      setLoadingIterations(false);
    }
  }, [currentSession]);

  const markGenerating = useCallback((iterationId: string) => {
    setGeneratingIds((prev) => (prev.includes(iterationId) ? prev : [...prev, iterationId]));
  }, []);

  const clearGenerating = useCallback((iterationId: string) => {
    setGeneratingIds((prev) => prev.filter((id) => id !== iterationId));
  }, []);

  useEffect(() => {
    loadProjectKnowledge();
    syncGenerationStatus();
  }, [loadProjectKnowledge, syncGenerationStatus, refreshTrigger]);

  useEffect(() => {
    if (currentSession && knowledgeList.length === 0 && !loading) {
      loadIterations();
    }
  }, [currentSession, knowledgeList.length, loading, currentIterationId, loadIterations]);

  // Generation lifecycle events keep the state in sync while the panel is mounted.
  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    const setupListeners = async () => {
      const registered = await Promise.all([
        listen<string>('knowledge_regeneration_started', (event) => {
          if (disposed) return;
          markGenerating(event.payload);
        }),
        listen<string>('knowledge_regeneration_completed', (event) => {
          if (disposed) return;
          clearGenerating(event.payload);
          // The backend rewrites knowledge on completion; pull the fresh copy.
          loadProjectKnowledge();
        }),
        listen<[string, string]>('knowledge_regeneration_failed', (event) => {
          if (disposed) return;
          const [iterationId, reason] = event.payload;
          clearGenerating(iterationId);
          // A failed run leaves the previous knowledge removed — reload so the
          // list matches what is actually on disk.
          loadProjectKnowledge();
          message.error("Knowledge generation failed: " + reason);
        }),
      ]);

      if (disposed) {
        registered.forEach((unlisten) => unlisten());
      } else {
        unlisteners.push(...registered);
      }
    };

    setupListeners();

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [markGenerating, clearGenerating, loadProjectKnowledge, message]);

  // Safety net: if a lifecycle event is ever missed, polling reconciles the
  // state instead of leaving the panel stuck on "Generating..." forever.
  useEffect(() => {
    if (generatingIds.length === 0) return;
    const timer = window.setInterval(syncGenerationStatus, 10000);
    return () => window.clearInterval(timer);
  }, [generatingIds.length, syncGenerationStatus]);

  // Find current iteration from iterations list
  const currentIterationInfo = currentIterationId 
    ? iterations.find(iter => iter.id === currentIterationId) 
    : null;

  const isGenerating = (iterationId: string) => generatingIds.includes(iterationId);
  const isAnyGenerating = generatingIds.length > 0;

  const handleGenerateKnowledge = async (iterationId: string) => {
    // Optimistic: reflect the click immediately, then let the backend confirm.
    markGenerating(iterationId);
    try {
      await invoke("gui_regenerate_knowledge", { iterationId });
      // Don't show success message here - wait for event from backend
      // The actual knowledge generation happens in background
    } catch (error) {
      console.error("[KnowledgePanel] Failed to start knowledge generation:", error);
      message.error("Failed to start generation: " + error);
      clearGenerating(iterationId);
    }
  };

  const handleViewDetail = (knowledge: Knowledge) => {
    setSelectedKnowledge(knowledge);
  };

  const formatDate = (dateString?: string): string => {
    if (!dateString) return "N/A";
    try {
      return new Date(dateString).toLocaleString("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" });
    } catch {
      return dateString;
    }
  };

  const filteredKnowledge = knowledgeList.filter((knowledge) => {
    if (!searchText.trim()) return true;
    const searchLower = searchText.toLowerCase();
    return (
      knowledge.title?.toLowerCase().includes(searchLower) ||
      knowledge.idea_summary?.toLowerCase().includes(searchLower) ||
      knowledge.design_summary?.toLowerCase().includes(searchLower) ||
      knowledge.plan_summary?.toLowerCase().includes(searchLower) ||
      knowledge.iteration_id?.toLowerCase().includes(searchLower)
    );
  });

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column", padding: "16px" }}>
      <Card size="small" style={{ marginBottom: "16px" }}>
        <Space orientation="vertical" style={{ width: "100%" }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <Space>
              <BookOutlined style={{ fontSize: "16px", color: "#1890ff" }} />
              <Text strong>Knowledge</Text>
              <Tag color="blue">{filteredKnowledge.length} iterations</Tag>
              {isAnyGenerating && (
                <Tag color="processing" icon={<LoadingOutlined spin />}>
                  Generating{generatingIds.length > 1 ? ` (${generatingIds.length})` : ""}…
                </Tag>
              )}
            </Space>
            <Button icon={<ReloadOutlined />} size="small" onClick={loadProjectKnowledge} loading={loading}>Refresh</Button>
          </div>
          <Input placeholder="Search knowledge..." prefix={<SearchOutlined />} value={searchText} onChange={(e) => setSearchText(e.target.value)} allowClear />
        </Space>
      </Card>

      <div style={{ flex: 1, overflow: "auto" }}>
        {loading ? (
          <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}><Spin size="large" /></div>
        ) : !currentSession ? (
          <Empty description="Please select a project" style={{ marginTop: "50px" }} />
        ) : filteredKnowledge.length === 0 ? (
          loadingIterations ? (
            <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%" }}><Spin size="large" /></div>
          ) : currentIterationInfo ? (
            // Current iteration is selected but has no knowledge - show generate option
            <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
              {isGenerating(currentIterationInfo.id) ? (
                // Generation is running (possibly resumed after a tab switch):
                // never offer a clickable Generate button here.
                <Card size="small" style={{ borderLeft: "3px solid var(--primary)" }}>
                  <div style={{ display: "flex", alignItems: "flex-start", gap: "12px" }}>
                    <LoadingOutlined spin style={{ fontSize: "18px", color: "var(--primary)", marginTop: "2px" }} />
                    <div style={{ flex: 1 }}>
                      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px" }}>
                        <Text strong>Generating knowledge for #{currentIterationInfo.number} {currentIterationInfo.title}</Text>
                        <Tag color="processing">In progress</Tag>
                      </div>
                      <Paragraph style={{ margin: 0, fontSize: "12px", color: "var(--text-secondary)" }}>
                        Reading artifacts and summarizing with the model. This can take a few minutes — the status is kept if you switch tabs.
                      </Paragraph>
                    </div>
                  </div>
                </Card>
              ) : (
                <>
                  <div style={{ padding: "12px", background: "var(--info-light)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid var(--info)", marginBottom: "8px" }}>
                    <Text style={{ color: "var(--primary)" }}>Current iteration has no knowledge. Generate knowledge for it:</Text>
                  </div>
                  <Card size="small" hoverable>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "8px" }}>
                      <div style={{ flex: 1 }}>
                        <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px" }}>
                          <Text strong>#{currentIterationInfo.number} {currentIterationInfo.title}</Text>
                          <Badge status="success" text="Completed" />
                        </div>
                        <Paragraph ellipsis={{ rows: 2 }} style={{ margin: 0, fontSize: "13px", color: "#888" }}>
                          {currentIterationInfo.description}
                        </Paragraph>
                      </div>
                      <Tooltip title={isAnyGenerating ? "Another generation is already running" : "Generate knowledge"}>
                        <Button 
                          type="primary" 
                          icon={<PlusOutlined />} 
                          size="small" 
                          onClick={() => handleGenerateKnowledge(currentIterationInfo.id)} 
                          loading={isGenerating(currentIterationInfo.id)}
                          disabled={isAnyGenerating}
                        >
                          Generate
                        </Button>
                      </Tooltip>
                    </div>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: "12px", color: "#666" }}>
                      <Space size="small"><span><ClockCircleOutlined /> {formatDate(currentIterationInfo.created_at)}</span></Space>
                    </div>
                  </Card>
                </>
              )}
            </div>
          ) : iterations.length > 0 ? (
            // No current iteration selected - show all completed iterations
            <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
              <div style={{ padding: "12px", background: "var(--success-light)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid var(--success)", marginBottom: "8px" }}>
                <Text style={{ color: "var(--success)" }}>No knowledge found. You can generate knowledge for completed iterations:</Text>
              </div>
              {iterations.map((iteration) => (
                <Card key={iteration.id} size="small" hoverable>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "8px" }}>
                    <div style={{ flex: 1 }}>
                      <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px" }}>
                        <Text strong>#{iteration.number} {iteration.title}</Text>
                        <Badge status="success" text="Completed" />
                        {isGenerating(iteration.id) && <Tag color="processing" icon={<LoadingOutlined spin />}>In progress</Tag>}
                      </div>
                      <Paragraph ellipsis={{ rows: 2 }} style={{ margin: 0, fontSize: "13px", color: "#888" }}>
                        {iteration.description}
                      </Paragraph>
                    </div>
                    <Tooltip title={isGenerating(iteration.id) ? "Generation in progress…" : isAnyGenerating ? "Another generation is already running" : "Generate knowledge"}>
                      <Button 
                        type="primary" 
                        icon={<PlusOutlined />} 
                        size="small" 
                        onClick={() => handleGenerateKnowledge(iteration.id)} 
                        loading={isGenerating(iteration.id)}
                        disabled={isGenerating(iteration.id) || isAnyGenerating}
                      >
                        {isGenerating(iteration.id) ? "Generating…" : "Generate"}
                      </Button>
                    </Tooltip>
                  </div>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: "12px", color: "#666" }}>
                    <Space size="small"><span><ClockCircleOutlined /> {formatDate(iteration.created_at)}</span></Space>
                  </div>
                </Card>
              ))}
            </div>
          ) : (
            <Empty description="No knowledge found. Complete an iteration first to generate knowledge." style={{ marginTop: "50px" }} />
          )
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            {filteredKnowledge.map((knowledge) => (
              <Card key={knowledge.iteration_id} size="small" hoverable style={{ cursor: "pointer" }} onClick={() => handleViewDetail(knowledge)}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "8px" }}>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "4px" }}>
                      <Text strong>{knowledge.title}</Text>
                      <Tag color="geekblue">{knowledge.iteration_id.slice(0, 8)}</Tag>
                      {isGenerating(knowledge.iteration_id) && <Tag color="processing" icon={<LoadingOutlined spin />}>Regenerating</Tag>}
                    </div>
                    <Paragraph ellipsis={{ rows: 2 }} style={{ margin: 0, fontSize: "13px", color: "#888" }}>
                      {knowledge.plan_summary || knowledge.design_summary || knowledge.idea_summary}
                    </Paragraph>
                  </div>
                  <Space size="small">
                    <Tooltip title="查看详情">
                      <Button type="text" icon={<EyeOutlined />} size="small" onClick={(e) => { e.stopPropagation(); handleViewDetail(knowledge); }} />
                    </Tooltip>
                    <Tooltip title={isGenerating(knowledge.iteration_id) ? "重新生成中…" : isAnyGenerating ? "已有生成任务在进行中" : "重新生成知识"}>
                      <Button type="text" icon={<ReloadOutlined />} size="small" onClick={(e) => { e.stopPropagation(); handleGenerateKnowledge(knowledge.iteration_id); }} loading={isGenerating(knowledge.iteration_id)} disabled={isGenerating(knowledge.iteration_id) || isAnyGenerating} />
                    </Tooltip>
                  </Space>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: "12px", color: "#666" }}>
                  <Space size="small"><span><ClockCircleOutlined /> {formatDate(knowledge.created_at)}</span></Space>
                  {knowledge.tech_stack && knowledge.tech_stack.length > 0 && (
                    <Space size="small">
                      {knowledge.tech_stack.slice(0, 2).map((tech, idx) => <Tag key={`tech-${idx}`} style={{ fontSize: "11px" }}>{tech}</Tag>)}
                    </Space>
                  )}
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>

      <Modal
        title={
          <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
            <BookOutlined style={{ fontSize: "18px", color: "var(--primary)" }} />
            <span style={{ fontSize: "16px", fontWeight: 600 }}>{selectedKnowledge?.title}</span>
            <Tag color="geekblue">{selectedKnowledge?.iteration_id?.slice(0, 8)}</Tag>
          </div>
        }
        open={!!selectedKnowledge}
        onCancel={() => setSelectedKnowledge(null)}
        footer={null}
        width={900}
        style={{ top: "5vh" }}
        styles={{ body: { padding: "20px", maxHeight: "75vh", overflow: "auto" } }}
      >
        {selectedKnowledge && (
          <Tabs
            defaultActiveKey="summary"
            items={[
              {
                key: "summary",
                label: <span><FileTextOutlined /> Summary</span>,
                children: (
                  <div style={{ padding: "12px 0" }}>
                    <Space orientation="vertical" style={{ width: "100%" }} size="middle">
                      {selectedKnowledge.idea_summary && (
                        <div style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid var(--success)" }}>
                          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "12px" }}>
                            <RocketOutlined style={{ color: "var(--success)", fontSize: "16px" }} />
                            <Title level={5} style={{ margin: 0, color: "var(--success)" }}>Idea Summary</Title>
                          </div>
                          <Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)" }}>{selectedKnowledge.idea_summary}</Paragraph>
                        </div>
                      )}
                      {selectedKnowledge.design_summary && (
                        <div style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid #722ed1" }}>
                          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "12px" }}>
                            <EyeOutlined style={{ color: "#722ed1", fontSize: "16px" }} />
                            <Title level={5} style={{ margin: 0, color: "#722ed1" }}>Design Summary</Title>
                          </div>
                          <Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)" }}>{selectedKnowledge.design_summary}</Paragraph>
                        </div>
                      )}
                      {selectedKnowledge.plan_summary && (
                        <div style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid var(--warning)" }}>
                          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "12px" }}>
                            <FileTextOutlined style={{ color: "var(--warning)", fontSize: "16px" }} />
                            <Title level={5} style={{ margin: 0, color: "var(--warning)" }}>Plan Summary</Title>
                          </div>
                          <Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)" }}>{selectedKnowledge.plan_summary}</Paragraph>
                        </div>
                      )}
                      {selectedKnowledge.code_structure && (
                        <div style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid var(--primary)" }}>
                          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "12px" }}>
                            <CodeOutlined style={{ color: "var(--primary)", fontSize: "16px" }} />
                            <Title level={5} style={{ margin: 0, color: "var(--primary)" }}>Code Structure</Title>
                          </div>
                          <Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)", whiteSpace: "pre-wrap" }}>{selectedKnowledge.code_structure}</Paragraph>
                        </div>
                      )}
                    </Space>
                  </div>
                ),
              },
              {
                key: "tech-stack",
                label: <span><CodeOutlined /> Tech Stack</span>,
                children: (
                  <div style={{ padding: "12px 0" }}>
                    {selectedKnowledge.tech_stack?.length > 0 ? (
                      <div style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)" }}>
                        <Text strong style={{ display: "block", marginBottom: "12px", fontSize: "14px" }}>Tech Stack</Text>
                        <div style={{ display: "flex", flexWrap: "wrap", gap: "8px" }}>
                          {selectedKnowledge.tech_stack.map((tech, idx) => <Tag key={`tech-${idx}`} color="blue" style={{ padding: "4px 12px", fontSize: "13px" }}>{tech}</Tag>)}
                        </div>
                      </div>
                    ) : <Empty description="No tech stack information" />}
                  </div>
                ),
              },
              {
                key: "decisions",
                label: <span><CheckCircleOutlined /> Key Decisions</span>,
                children: (
                  <div style={{ padding: "12px 0" }}>
                    {selectedKnowledge.key_decisions?.length > 0 ? (
                      <Timeline style={{ marginTop: "12px" }} items={selectedKnowledge.key_decisions.map((decision) => ({ children: <div style={{ padding: "12px", background: "var(--bg-base)", borderRadius: "3px", border: "1px solid var(--border-color)" }}><Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)" }}>{decision}</Paragraph></div> }))} />
                    ) : <Empty description="No key decisions recorded" />}
                  </div>
                ),
              },
              {
                key: "patterns",
                label: <span><BookOutlined /> Design Patterns</span>,
                children: (
                  <div style={{ padding: "12px 0" }}>
                    {selectedKnowledge.key_patterns?.length > 0 ? (
                      <Space orientation="vertical" style={{ width: "100%" }} size="middle">
                        {selectedKnowledge.key_patterns.map((pattern, idx) => (
                          <div key={`pattern-${idx}`} style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid #722ed1" }}>
                            <Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)" }}>{pattern}</Paragraph>
                          </div>
                        ))}
                      </Space>
                    ) : <Empty description="No design patterns recorded" />}
                  </div>
                ),
              },
              {
                key: "issues",
                label: <span><CloseCircleOutlined /> Known Issues</span>,
                children: (
                  <div style={{ padding: "12px 0" }}>
                    {selectedKnowledge.known_issues?.length > 0 ? (
                      <Space orientation="vertical" style={{ width: "100%" }} size="middle">
                        {selectedKnowledge.known_issues.map((issue, idx) => (
                          <div key={`issue-${idx}`} style={{ padding: "16px", background: "var(--bg-elevated)", borderRadius: "3px", border: "1px solid var(--border-light)", borderLeft: "3px solid var(--error)" }}>
                            <div style={{ display: "flex", alignItems: "flex-start", gap: "8px" }}>
                              <CloseCircleOutlined style={{ color: "var(--error)", marginTop: "6px" }} />
                              <Paragraph style={{ margin: 0, lineHeight: "1.8", color: "var(--text-primary)", flex: 1 }}>{issue}</Paragraph>
                            </div>
                          </div>
                        ))}
                      </Space>
                    ) : <Empty description="No known issues recorded" />}
                  </div>
                ),
              },
            ]}
          />
        )}
      </Modal>
    </div>
  );
};

export default KnowledgePanel;
