# 方案 01: 顶层项目管理器

**版本**: 1.0  
**创建日期**: 2026-02-02  
**Phase**: Phase 0  
**工作量**: 40h  
**优先级**: P0 (必须)

---

## 📋 概述

### 1.1 需求背景

**当前问题**:
- Cowork GUI 只读取程序启动路径下的 `.cowork`
- 无法统一管理多个 Cowork 项目
- 用户需要切换目录才能管理不同项目
- 缺少项目列表和项目管理功能

**目标**:
- 在系统应用配置目录（AppData）管理所有项目
- 主界面新增 "Projects" Tab，统一管理
- 支持项目创建、打开、删除等操作
- Open 操作在新窗口打开项目，避免对现有实现改动

### 1.2 设计原则

- ✅ 跨平台支持（Windows/macOS/Linux）
- ✅ 与现有实现解耦
- ✅ 支持新窗口打开项目
- ✅ 项目信息持久化存储
- ✅ 快速切换项目

---

## 🏗️ 技术方案

### 2.1 存储位置

**跨平台路径**:
```rust
// Windows
%APPDATA%\CoworkCreative\project_registry.json

// macOS
~/Library/Application Support/CoworkCreative/project_registry.json

// Linux
~/.config/cowork-creative/project_registry.json
```

### 2.2 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRegistry {
    pub schema_version: String,
    pub projects: Vec<ProjectRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub workspace_path: String,
    pub created_at: DateTime<Utc>,
    pub last_opened_at: Option<DateTime<Utc>>,
    pub status: ProjectStatus,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub session_count: usize,
    pub last_session_id: Option<String>,
    pub technology_stack: Vec<String>,
    pub project_type: String,
}
```

### 2.3 Tauri 命令

```rust
// 注册项目
#[tauri::command]
pub async fn register_project(
    workspace_path: String,
    name: String,
    description: Option<String>,
) -> Result<String, String>

// 获取所有项目
#[tauri::command]
pub async fn get_all_projects() -> Result<Vec<ProjectRecord>, String>

// 删除项目
#[tauri::command]
pub async fn delete_project(
    project_id: String,
    delete_files: bool,
) -> Result<(), String>

// 更新项目信息
#[tauri::command]
pub async fn update_project(
    project_id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), String>

// 打开项目（新窗口）
#[tauri::command]
pub async fn open_project(
    project_id: String,
) -> Result<(), String>

// 自动注册当前项目
#[tauri::command]
pub async fn auto_register_current_project() -> Result<Option<ProjectRecord>, String>
```

### 2.4 新窗口打开实现

```rust
#[tauri::command]
pub async fn open_project(project_id: String) -> Result<(), String> {
    let registry = load_project_registry()?;
    
    let project = registry.projects.iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    // 更新最后打开时间
    update_last_opened(&project_id)?;
    
    // 获取当前可执行文件路径
    let exe_path = std::env::current_exe()?;
    
    // 启动新进程
    std::process::Command::new(exe_path)
        .arg("--workspace")
        .arg(&project.workspace_path)
        .spawn()
        .map_err(|e| format!("Failed to open project: {}", e))?;
    
    Ok(())
}
```

### 2.5 自动注册

```rust
#[tauri::command]
pub async fn auto_register_current_project() -> Result<Option<ProjectRecord>, String> {
    let current_dir = std::env::current_dir()?;
    
    // 检查是否是 Cowork 项目
    if !is_cowork_project(&current_dir)? {
        return Ok(None);
    }
    
    // 检查是否已注册
    let registry = load_project_registry()?;
    let workspace_path = current_dir.to_string_lossy().to_string();
    
    if registry.projects.iter().any(|p| p.workspace_path == workspace_path) {
        return Ok(None);
    }
    
    // 读取项目信息
    let project_name = extract_project_name(&current_dir)?;
    let project_id = format!("proj-{}", chrono::Utc::now().timestamp_millis());
    
    let project = ProjectRecord {
        project_id: project_id.clone(),
        name: project_name,
        description: None,
        workspace_path,
        created_at: chrono::Utc::now(),
        last_opened_at: None,
        status: ProjectStatus::Active,
        metadata: ProjectMetadata {
            session_count: count_sessions(&current_dir)?,
            last_session_id: get_last_session_id(&current_dir)?,
            technology_stack: detect_tech_stack(&current_dir)?,
            project_type: detect_project_type(&current_dir)?,
        },
    };
    
    // 保存到注册表
    save_project(&project).await?;
    
    Ok(Some(project))
}
```

---

## 🎨 前端实现

### 3.1 ProjectsPanel 组件

```jsx
// projects/ProjectsPanel.jsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, Button, Modal, Input, Tag, Dropdown, message } from 'antd';
import { 
  FolderOpenOutlined, 
  DeleteOutlined, 
  EditOutlined, 
  PlusOutlined 
} from '@ant-design/icons';

const ProjectsPanel = () => {
  const [projects, setProjects] = useState([]);
  const [loading, setLoading] = useState(false);
  const [showAddModal, setShowAddModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [selectedProject, setSelectedProject] = useState(null);
  const [newProjectName, setNewProjectName] = useState('');
  const [newProjectPath, setNewProjectPath] = useState('');

  const loadProjects = async () => {
    setLoading(true);
    try {
      const data = await invoke('get_all_projects');
      setProjects(data);
    } catch (error) {
      message.error('Failed to load projects');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProjects();
  }, []);

  const handleAddProject = async () => {
    if (!newProjectName.trim() || !newProjectPath.trim()) {
      message.warning('Please enter project name and path');
      return;
    }
    
    try {
      await invoke('register_project', {
        workspace_path: newProjectPath,
        name: newProjectName,
        description: '',
      });
      message.success('Project registered');
      setShowAddModal(false);
      loadProjects();
    } catch (error) {
      message.error('Failed to register project: ' + error);
    }
  };

  const handleDeleteProject = async (projectId) => {
    Modal.confirm({
      title: 'Delete Project',
      content: 'Do you want to delete only the record or also delete all project files?',
      okText: 'Delete Record Only',
      okType: 'default',
      cancelText: 'Delete All',
      onOk: async () => {
        try {
          await invoke('delete_project', { project_id: projectId, delete_files: false });
          message.success('Project record deleted');
          loadProjects();
        } catch (error) {
          message.error('Failed to delete project');
        }
      },
      onCancel: async () => {
        try {
          await invoke('delete_project', { project_id: projectId, delete_files: true });
          message.success('Project deleted');
          loadProjects();
        } catch (error) {
          message.error('Failed to delete project');
        }
      },
    });
  };

  const handleOpenProject = async (projectId) => {
    try {
      await invoke('open_project', { project_id: projectId });
      message.info('Opening project in new window...');
    } catch (error) {
      message.error('Failed to open project');
    }
  };

  return (
    <div className="projects-panel">
      <div style={{ marginBottom: '20px', display: 'flex', justifyContent: 'space-between' }}>
        <h2>Projects</h2>
        <Button type="primary" icon={<PlusOutlined />} onClick={() => setShowAddModal(true)}>
          Add Project
        </Button>
      </div>

      {loading ? (
        <div style={{ textAlign: 'center', padding: '40px' }}>Loading...</div>
      ) : projects.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '40px', color: '#888' }}>
          No projects yet. Register your first Cowork project.
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '16px' }}>
          {projects.map(project => (
            <Card
              key={project.project_id}
              hoverable
              actions={[
                <Button 
                  type="link" 
                  icon={<FolderOpenOutlined />} 
                  onClick={() => handleOpenProject(project.project_id)}
                >
                  Open
                </Button>,
                <Button 
                  type="link" 
                  icon={<EditOutlined />} 
                  onClick={() => { setSelectedProject(project); setShowEditModal(true); }}
                >
                  Edit
                </Button>,
                <Button 
                  type="link" 
                  danger 
                  icon={<DeleteOutlined />} 
                  onClick={() => handleDeleteProject(project.project_id)}
                >
                  Delete
                </Button>,
              ]}
            >
              <Card.Meta
                title={project.name}
                description={
                  <div>
                    <div style={{ marginBottom: '8px', color: '#666' }}>
                      {project.description || 'No description'}
                    </div>
                    <div style={{ fontSize: '12px', color: '#999' }}>
                      {project.workspace_path}
                    </div>
                  </div>
                }
              />
              <div style={{ marginTop: '12px' }}>
                <Tag color={project.status === 'Active' ? 'green' : 'default'}>
                  {project.status}
                </Tag>
                <span style={{ marginLeft: '8px', fontSize: '12px', color: '#999' }}>
                  {project.metadata.session_count} sessions
                </span>
              </div>
            </Card>
          ))}
        </div>
      )}

      {/* Add Project Modal */}
      <Modal
        title="Add Project"
        visible={showAddModal}
        onOk={handleAddProject}
        onCancel={() => setShowAddModal(false)}
      >
        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '8px' }}>Project Name:</label>
          <Input 
            value={newProjectName}
            onChange={(e) => setNewProjectName(e.target.value)}
            placeholder="Enter project name"
          />
        </div>
        <div>
          <label style={{ display: 'block', marginBottom: '8px' }}>Workspace Path:</label>
          <Input 
            value={newProjectPath}
            onChange={(e) => setNewProjectPath(e.target.value)}
            placeholder="Enter path to Cowork project"
          />
        </div>
      </Modal>
    </div>
  );
};

export default ProjectsPanel;
```

---

## 📅 实施计划

### 4.1 任务分解 (40h)

#### 后端实现 (24h)
- [ ] ProjectRegistry 数据结构 (2h)
- [ ] 跨平台存储路径 (2h)
- [ ] 注册项目命令 (4h)
- [ ] 获取项目列表命令 (2h)
- [ ] 删除项目命令 (4h)
- [ ] 更新项目命令 (3h)
- [ ] 打开项目命令 (3h)
- [ ] 自动注册命令 (4h)

#### 前端实现 (12h)
- [ ] ProjectsPanel 组件 (4h)
- [ ] 项目列表展示 (2h)
- [ ] 添加项目功能 (2h)
- [ ] 编辑项目功能 (2h)
- [ ] 删除项目功能 (2h)
- [ ] 打开项目功能 (2h)

#### 测试与优化 (4h)
- [ ] 跨平台测试 (2h)
- [ ] 性能测试 (1h)
- [ ] 错误处理测试 (1h)

---

## 🎯 验收标准

### 功能验收
- ✅ 能够注册 Cowork 项目
- ✅ 能够查看项目列表
- ✅ 能够打开项目（新窗口）
- ✅ 能够编辑项目信息
- ✅ 能够删除项目（可选删除文件）
- ✅ 自动注册当前项目

### 技术验收
- ✅ 跨平台兼容（Windows/macOS/Linux）
- ✅ 项目信息持久化存储
- ✅ 新窗口打开正常工作
- ✅ 不影响现有项目使用习惯

---

**文档版本**: 1.0  
**创建时间**: 2026-02-02  
**Phase**: Phase 0  
**工作量**: 40h