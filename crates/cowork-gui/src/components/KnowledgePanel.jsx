import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Card,
  Input,
  Button,
  Tag,
  Empty,
  Spin,
  Modal,
  Tabs,
  Typography,
  Space,
  Divider,
  Timeline,
  message,
} from "antd";
import {
  SearchOutlined,
  EyeOutlined,
  BookOutlined,
  ClockCircleOutlined,
  ReloadOutlined,
  RocketOutlined,
  FileTextOutlined,
  CodeOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
} from "@ant-design/icons";

const { TextArea } = Input;
const { Option } = Input;
const { Text, Paragraph, Title } = Typography;

function KnowledgePanel({ currentSession }) {
  const [knowledgeList, setKnowledgeList] = useState([]);
  const [loading, setLoading] = useState(false);
  const [regenerating, setRegenerating] = useState(null);
  const [selectedKnowledge, setSelectedKnowledge] = useState(null);
  const [searchText, setSearchText] = useState("");

  useEffect(() => {
    loadProjectKnowledge();
  }, [currentSession]);

  const loadProjectKnowledge = async () => {
    if (!currentSession) {
      setKnowledgeList([]);
      return;
    }

    setLoading(true);
    try {
      const result = await invoke("gui_get_project_knowledge", {
        projectId: currentSession,
      });
      setKnowledgeList(result.knowledge_list || []);
    } catch (error) {
      console.error(
        "[KnowledgePanel] Failed to load project knowledge:",
        error,
      );
      message.error("Failed to load project knowledge: " + error);
      setKnowledgeList([]);
    } finally {
      setLoading(false);
    }
  };

  const handleRegenerateKnowledge = async (iterationId) => {
    setRegenerating(iterationId);
    try {
      await invoke("gui_regenerate_knowledge", { iterationId });
      message.success("Knowledge regenerated successfully");
      loadProjectKnowledge();
    } catch (error) {
      console.error("[KnowledgePanel] Failed to regenerate knowledge:", error);
      message.error("Failed to regenerate knowledge: " + error);
    } finally {
      setRegenerating(null);
    }
  };

  const handleViewDetail = (knowledge) => {
    setSelectedKnowledge(knowledge);
  };

  const formatDate = (dateString) => {
    try {
      const date = new Date(dateString);
      return date.toLocaleString("zh-CN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch (e) {
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
    <div
      style={{
        height: "100%",
        display: "flex",
        flexDirection: "column",
        padding: "16px",
      }}
    >
      <Card size="small" style={{ marginBottom: "16px" }}>
        <Space direction="vertical" style={{ width: "100%" }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
            }}
          >
            <Space>
              <BookOutlined style={{ fontSize: "16px", color: "#1890ff" }} />
              <Text strong>项目知识</Text>
              <Tag color="blue">{filteredKnowledge.length} 个迭代</Tag>
            </Space>
            <Button
              icon={<ReloadOutlined />}
              size="small"
              onClick={loadProjectKnowledge}
              loading={loading}
            >
              刷新
            </Button>
          </div>
          <Input
            placeholder="搜索知识..."
            prefix={<SearchOutlined />}
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            allowClear
          />
        </Space>
      </Card>

      <div style={{ flex: 1, overflow: "auto" }}>
        {loading ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
            }}
          >
            <Spin size="large" />
          </div>
        ) : !currentSession ? (
          <Empty description="请先选择一个项目" style={{ marginTop: "50px" }} />
        ) : filteredKnowledge.length === 0 ? (
          <Empty description="暂无项目知识" style={{ marginTop: "50px" }} />
        ) : (
          <div
            style={{ display: "flex", flexDirection: "column", gap: "12px" }}
          >
            {filteredKnowledge.map((knowledge) => (
              <Card
                key={knowledge.iteration_id}
                size="small"
                hoverable
                style={{ cursor: "pointer" }}
                onClick={() => handleViewDetail(knowledge)}
              >
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "flex-start",
                    marginBottom: "8px",
                  }}
                >
                  <div style={{ flex: 1 }}>
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "8px",
                        marginBottom: "4px",
                      }}
                    >
                      <Text strong>{knowledge.title}</Text>
                      <Tag color="geekblue">
                        {knowledge.iteration_id.slice(0, 8)}
                      </Tag>
                    </div>
                    <Paragraph
                      ellipsis={{ rows: 2 }}
                      style={{ margin: 0, fontSize: "13px", color: "#888" }}
                    >
                      {knowledge.plan_summary ||
                        knowledge.design_summary ||
                        knowledge.idea_summary}
                    </Paragraph>
                  </div>
                  <Space size="small">
                    <Button
                      type="text"
                      icon={<EyeOutlined />}
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleViewDetail(knowledge);
                      }}
                    />
                    <Button
                      type="text"
                      icon={<ReloadOutlined />}
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRegenerateKnowledge(knowledge.iteration_id);
                      }}
                      loading={regenerating === knowledge.iteration_id}
                    />
                  </Space>
                </div>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    fontSize: "12px",
                    color: "#666",
                  }}
                >
                  <Space size="small">
                    <span>
                      <ClockCircleOutlined /> {formatDate(knowledge.created_at)}
                    </span>
                  </Space>
                  {knowledge.tech_stack && knowledge.tech_stack.length > 0 && (
                    <Space size="small">
                      {knowledge.tech_stack.slice(0, 2).map((tech, idx) => (
                        <Tag key={`tech-${idx}`} style={{ fontSize: "11px" }}>
                          {tech}
                        </Tag>
                      ))}
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
            <BookOutlined style={{ fontSize: "18px", color: "#1890ff" }} />
            <span style={{ fontSize: "16px", fontWeight: 600 }}>
              {selectedKnowledge?.title}
            </span>
            <Tag color="geekblue">
              {selectedKnowledge?.iteration_id?.slice(0, 8)}
            </Tag>
          </div>
        }
        open={!!selectedKnowledge}
        onCancel={() => setSelectedKnowledge(null)}
        footer={null}
        width={900}
        styles={{
          body: { padding: "24px" },
        }}
      >
        {selectedKnowledge && (
          <div style={{ maxHeight: "75vh", overflow: "auto" }}>
            <Tabs
              defaultActiveKey="summary"
              items={[
                {
                  key: "summary",
                  label: (
                    <span>
                      <FileTextOutlined /> 总结
                    </span>
                  ),
                  children: (
                    <div style={{ padding: "12px 0" }}>
                      <Space
                        direction="vertical"
                        style={{ width: "100%" }}
                        size="large"
                      >
                        {selectedKnowledge.idea_summary && (
                          <div
                            style={{
                              padding: "16px",
                              backgroundColor: "#f6ffed",
                              borderRadius: "8px",
                              border: "1px solid #b7eb8f",
                            }}
                          >
                            <div
                              style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "8px",
                                marginBottom: "12px",
                              }}
                            >
                              <RocketOutlined
                                style={{ color: "#52c41a", fontSize: "16px" }}
                              />
                              <Title
                                level={5}
                                style={{ margin: 0, color: "#52c41a" }}
                              >
                                Idea 总结
                              </Title>
                            </div>
                            <Paragraph
                              style={{
                                margin: 0,
                                lineHeight: "1.8",
                                color: "#333",
                              }}
                            >
                              {selectedKnowledge.idea_summary}
                            </Paragraph>
                          </div>
                        )}
                        {selectedKnowledge.design_summary && (
                          <div
                            style={{
                              padding: "16px",
                              backgroundColor: "#f9f0ff",
                              borderRadius: "8px",
                              border: "1px solid #d3adf7",
                            }}
                          >
                            <div
                              style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "8px",
                                marginBottom: "12px",
                              }}
                            >
                              <EyeOutlined
                                style={{ color: "#722ed1", fontSize: "16px" }}
                              />
                              <Title
                                level={5}
                                style={{ margin: 0, color: "#722ed1" }}
                              >
                                Design 总结
                              </Title>
                            </div>
                            <Paragraph
                              style={{
                                margin: 0,
                                lineHeight: "1.8",
                                color: "#333",
                              }}
                            >
                              {selectedKnowledge.design_summary}
                            </Paragraph>
                          </div>
                        )}
                        {selectedKnowledge.plan_summary && (
                          <div
                            style={{
                              padding: "16px",
                              backgroundColor: "#fff7e6",
                              borderRadius: "8px",
                              border: "1px solid #ffd591",
                            }}
                          >
                            <div
                              style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "8px",
                                marginBottom: "12px",
                              }}
                            >
                              <FileTextOutlined
                                style={{ color: "#fa8c16", fontSize: "16px" }}
                              />
                              <Title
                                level={5}
                                style={{ margin: 0, color: "#fa8c16" }}
                              >
                                Plan 总结
                              </Title>
                            </div>
                            <Paragraph
                              style={{
                                margin: 0,
                                lineHeight: "1.8",
                                color: "#333",
                              }}
                            >
                              {selectedKnowledge.plan_summary}
                            </Paragraph>
                          </div>
                        )}
                        {selectedKnowledge.code_structure && (
                          <div
                            style={{
                              padding: "16px",
                              backgroundColor: "#e6f7ff",
                              borderRadius: "8px",
                              border: "1px solid #91d5ff",
                            }}
                          >
                            <div
                              style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "8px",
                                marginBottom: "12px",
                              }}
                            >
                              <CodeOutlined
                                style={{ color: "#1890ff", fontSize: "16px" }}
                              />
                              <Title
                                level={5}
                                style={{ margin: 0, color: "#1890ff" }}
                              >
                                代码结构
                              </Title>
                            </div>
                            <Paragraph
                              style={{
                                margin: 0,
                                lineHeight: "1.8",
                                color: "#333",
                                whiteSpace: "pre-wrap",
                              }}
                            >
                              {selectedKnowledge.code_structure}
                            </Paragraph>
                          </div>
                        )}
                      </Space>
                    </div>
                  ),
                },
                {
                  key: "tech-stack",
                  label: (
                    <span>
                      <CodeOutlined /> 技术栈
                    </span>
                  ),
                  children: (
                    <div style={{ padding: "12px 0" }}>
                      <Space
                        direction="vertical"
                        style={{ width: "100%" }}
                        size="middle"
                      >
                        {selectedKnowledge.tech_stack?.length > 0 ? (
                          <div
                            style={{
                              padding: "16px",
                              backgroundColor: "#f5f5f5",
                              borderRadius: "8px",
                            }}
                          >
                            <Text
                              strong
                              style={{
                                display: "block",
                                marginBottom: "12px",
                                fontSize: "14px",
                              }}
                            >
                              技术栈
                            </Text>
                            <div
                              style={{
                                display: "flex",
                                flexWrap: "wrap",
                                gap: "8px",
                              }}
                            >
                              {selectedKnowledge.tech_stack.map((tech, idx) => (
                                <Tag
                                  key={`tech-${idx}`}
                                  color="blue"
                                  style={{
                                    padding: "4px 12px",
                                    fontSize: "13px",
                                  }}
                                >
                                  {tech}
                                </Tag>
                              ))}
                            </div>
                          </div>
                        ) : (
                          <Empty description="暂无技术栈信息" />
                        )}
                      </Space>
                    </div>
                  ),
                },
                {
                  key: "decisions",
                  label: (
                    <span>
                      <CheckCircleOutlined /> 关键决策
                    </span>
                  ),
                  children: (
                    <div style={{ padding: "12px 0" }}>
                      {selectedKnowledge.key_decisions?.length > 0 ? (
                        <Timeline
                          style={{ marginTop: "12px" }}
                          items={selectedKnowledge.key_decisions.map(
                            (decision, idx) => ({
                              children: (
                                <div
                                  style={{
                                    padding: "12px",
                                    backgroundColor: "#fff",
                                    borderRadius: "6px",
                                    border: "1px solid #e8e8e8",
                                  }}
                                >
                                  <Paragraph
                                    style={{
                                      margin: 0,
                                      lineHeight: "1.8",
                                      color: "#333",
                                    }}
                                  >
                                    {decision}
                                  </Paragraph>
                                </div>
                              ),
                            }),
                          )}
                        />
                      ) : (
                        <Empty description="无关键决策记录" />
                      )}
                    </div>
                  ),
                },
                {
                  key: "patterns",
                  label: (
                    <span>
                      <BookOutlined /> 设计模式
                    </span>
                  ),
                  children: (
                    <div style={{ padding: "12px 0" }}>
                      {selectedKnowledge.key_patterns?.length > 0 ? (
                        <Space
                          direction="vertical"
                          style={{ width: "100%" }}
                          size="middle"
                        >
                          {selectedKnowledge.key_patterns.map(
                            (pattern, idx) => (
                              <div
                                key={`pattern-${idx}`}
                                style={{
                                  padding: "16px",
                                  backgroundColor: "#f9f0ff",
                                  borderRadius: "8px",
                                  border: "1px solid #d3adf7",
                                }}
                              >
                                <Paragraph
                                  style={{
                                    margin: 0,
                                    lineHeight: "1.8",
                                    color: "#333",
                                  }}
                                >
                                  {pattern}
                                </Paragraph>
                              </div>
                            ),
                          )}
                        </Space>
                      ) : (
                        <Empty description="无设计模式记录" />
                      )}
                    </div>
                  ),
                },
                {
                  key: "issues",
                  label: (
                    <span>
                      <CloseCircleOutlined /> 已知问题
                    </span>
                  ),
                  children: (
                    <div style={{ padding: "12px 0" }}>
                      {selectedKnowledge.known_issues?.length > 0 ? (
                        <Space
                          direction="vertical"
                          style={{ width: "100%" }}
                          size="middle"
                        >
                          {selectedKnowledge.known_issues.map((issue, idx) => (
                            <div
                              key={`issue-${idx}`}
                              style={{
                                padding: "16px",
                                backgroundColor: "#fff2f0",
                                borderRadius: "8px",
                                border: "1px solid #ffccc7",
                              }}
                            >
                              <div
                                style={{
                                  display: "flex",
                                  alignItems: "flex-start",
                                  gap: "8px",
                                }}
                              >
                                <CloseCircleOutlined
                                  style={{ color: "#ff4d4f", marginTop: "6px" }}
                                />
                                <Paragraph
                                  style={{
                                    margin: 0,
                                    lineHeight: "1.8",
                                    color: "#333",
                                    flex: 1,
                                  }}
                                >
                                  {issue}
                                </Paragraph>
                              </div>
                            </div>
                          ))}
                        </Space>
                      ) : (
                        <Empty description="无已知问题记录" />
                      )}
                    </div>
                  ),
                },
              ]}
            />
          </div>
        )}
      </Modal>
    </div>
  );
}

export default KnowledgePanel;
