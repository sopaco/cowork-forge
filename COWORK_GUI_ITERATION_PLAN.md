# Cowork Creative Studio 发展与迭代计划

**版本**: 1.1  
**创建日期**: 2026-02-02  
**最后更新**: 2026-02-02  
**目标定位**: 面向中小型应用和系统的创意生产力工具，提供全流程全角色Agent驱动的应用创作与迭代能力

---

## 📋 执行摘要

Cowork Creative Studio 是一款基于 AI 智能体的创意生产力工具，当前已完成基础架构搭建和核心功能实现。本计划旨在将其打造成为面向中小型应用和系统的专业创作平台，在产品交互层面进行系统性升级，提供更流畅、更专业、更高效的用户体验。

### 核心目标

1. **提升产品交互体验**：优化 UI/UX，降低学习成本，提高操作效率
2. **增强创作流程可视化**：让用户清晰了解 AI 智能体的工作过程和决策依据
3. **强化实时协作能力**：提供更流畅的人机交互界面和实时反馈机制
4. **构建专业创作环境**：提供类似专业 IDE 的开发体验
5. **支持多项目并行管理**：满足多项目、多场景的开发需求
6. **降低迭代成本**：通过智能调度 Agent，简化 New/Modify 流程，支持项目完成后继续交互

### 当前状态评估

| 维度 | 完成度 | 评级 | 说明 |
|------|--------|------|------|
| **核心功能** | 85% | ✅ 良好 | 基础功能已实现，工作流完整 |
| **UI/UX 设计** | 60% | ⚠️ 需改进 | 功能可用但体验不专业 |
| **实时反馈** | 70% | ⚠️ 需优化 | 事件流实现但部分功能不完善 |
| **代码编辑** | 85% | ✅ 良好 | Monaco Editor 集成完善 |
| **预览能力** | 90% | ✅ 优秀 | 预览服务器功能完整 |
| **项目管理** | 40% | ⚠️ 需重构 | 仅支持单项目，缺少顶层项目管理 |
| **迭代流程** | 50% | ⚠️ 需优化 | New/Modify 使用成本高，缺少智能调度 |
| **错误处理** | 60% | ⚠️ 需改进 | 错误提示不友好 |
| **性能优化** | 65% | ⚠️ 需优化 | 大项目性能有问题 |

**综合评分**: 68/100 (良好但有明显改进空间)

### 核心问题识别

**问题1：缺少顶层项目管理**
- 当前只能读取启动路径下的 `.cowork` 目录
- 无法跨目录管理多个项目
- 缺少项目列表、打开、删除等基础操作
- 用户需要在不同的工作目录间切换，体验不流畅

**问题2：迭代流程使用成本高**
- New 创建的项目在 Completed 后无法继续聊天交互
- 需要手动选择 Resume/Modify 操作，学习成本高
- 缺少智能 Agent 来自动调度修改模式
- 项目完成后继续开发的流程不够自然

**影响范围**:
- 用户体验：需要频繁切换目录，操作繁琐
- 学习曲线：New/Modify 流程不直观，新用户难以理解
- 开发效率：无法快速继续开发已完成的项目

---

## 🎯 产品定位与用户画像

### 产品定位

**Cowork Creative Studio** 是一款面向中小型应用和系统的创意生产力工具，通过 AI 智能体协作，为用户提供全流程、全角色的应用创作与迭代能力。

**核心价值主张**：
- 🎨 **从创意到交付**：一站式完成从想法到可运行应用的完整创作流程
- 🤖 **AI 驱动的协作团队**：内置 8 个专业 AI 角色，模拟真实开发团队协作
- ⚡ **快速迭代**：支持增量修改、版本回退，灵活应对需求变化
- 🔒 **安全可控**：人机协作验证，确保输出质量和方向正确
- 📁 **多项目管理**：统一管理所有项目，快速切换，无需切换目录
- 🔄 **智能迭代**：项目完成后继续交互，自动调度修改模式

### 目标用户

#### 1. 独立开发者 / 创业者
**特征**：
- 需要快速将创意转化为可验证的原型
- 时间紧迫，追求开发效率
- 需要完整的技术栈支持

**核心诉求**：
- 避免从零开始的架构设计负担
- 快速生成可运行的初始代码
- 支持快速迭代和功能增强

**使用场景**：
- 构建个人项目原型
- 验证商业模式
- 快速发布 MVP

#### 2. 技术创业者 / 小团队
**特征**：
- 需要管理多个项目
- 追求开发效率和代码质量
- 需要团队协作能力

**核心诉求**：
- 多项目并行管理
- 统一的开发工作流
- 可追溯的变更历史

**使用场景**：
- 产品快速迭代
- 功能增强开发
- 技术债务管理

#### 3. 产品经理 / 设计师
**特征**：
- 需要快速验证产品想法
- 关注用户体验和功能完整性
- 需要与开发团队协作

**核心诉求**：
- 可视化需求文档
- 快速查看实现效果
- 易于理解的开发进度

**使用场景**：
- 产品原型验证
- 需求文档管理
- 功能演示

---

## 📊 当前架构与功能分析

### 现有技术架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    Cowork Creative Studio                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐     ┌─────────────────┐                   │
│  │   前端层 (React) │     │  后端层 (Rust)  │                   │
│  │  - App.jsx      │────▶│  - lib.rs       │                   │
│  │  - 组件系统     │     │  - gui_commands │                   │
│  │  - Ant Design   │     │  - preview_server│                  │
│  │  - Monaco       │     │  - project_runner│                  │
│  └─────────────────┘     └─────────────────┘                   │
│           │                       │                            │
│           ▼                       ▼                            │
│  ┌─────────────────┐     ┌─────────────────┐                   │
│  │  事件通信       │────▶│  cowork-core    │                   │
│  │  - agent_event  │     │  - pipeline     │                   │
│  │  - hitl_request │     │  - agents       │                   │
│  │  - session_state│     │  - storage      │                   │
│  └─────────────────┘     └─────────────────┘                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 已实现功能清单

| 功能模块 | 子功能 | 状态 | 完成度 |
|---------|--------|------|--------|
| **会话管理** | 创建项目 | ✅ | 100% |
| | 恢复会话 | ✅ | 100% |
| | 修改项目 | ✅ | 100% |
| | 回退版本 | ✅ | 100% |
| | 会话列表 | ✅ | 100% |
| **工件查看** | Idea 查看 | ✅ | 100% |
| | PRD 查看 | ✅ | 100% |
| | Features 查看 | ✅ | 100% |
| | Design 查看 | ✅ | 100% |
| | Plan 查看 | ✅ | 100% |
| | Delivery 查看 | ✅ | 100% |
| **代码编辑** | 文件树导航 | ✅ | 95% |
| | 代码编辑 | ✅ | 95% |
| | 语法高亮 | ✅ | 95% |
| | 文件保存 | ✅ | 95% |
| | 多语言支持 | ✅ | 90% |
| **实时预览** | 启动预览 | ✅ | 100% |
| | 停止预览 | ✅ | 100% |
| | 刷新预览 | ✅ | 100% |
| | iframe 嵌入 | ✅ | 100% |
| **项目运行** | 启动项目 | ✅ | 100% |
| | 停止项目 | ✅ | 100% |
| | 日志显示 | ⚠️ | 70% |
| | 实时日志流 | ❌ | 0% |
| **HITL 交互** | 输入请求 | ✅ | 100% |
| | 选项选择 | ✅ | 100% |
| | 反馈提交 | ✅ | 100% |
| **实时事件** | Agent 思考 | ✅ | 100% |
| | Agent 输出 | ✅ | 100% |
| | 阶段切换 | ✅ | 100% |
| | 错误通知 | ✅ | 100% |

### 待实现功能清单

| 功能模块 | 子功能 | 优先级 | 预计工时 |
|---------|--------|--------|---------|
| **实时日志流** | 日志流式推送 | P0 | 8h |
| | 日志级别着色 | P1 | 4h |
| | 日志搜索/过滤 | P1 | 6h |
| **代码编辑增强** | 代码格式化 | P1 | 6h |
| | 搜索/替换 | P1 | 4h |
| | Git Diff 查看 | P2 | 12h |
| | 代码导航 | P2 | 8h |
| **预览增强** | 设备模拟 | P2 | 8h |
| | 网络控制 | P2 | 6h |
| | 控制台查看 | P2 | 4h |
| **项目管理** | 项目模板 | P1 | 12h |
| | 项目导入/导出 | P2 | 10h |
| | 项目配置管理 | P2 | 8h |
| **协作功能** | 多用户协作 | P2 | 20h |
| | 分享功能 | P2 | 8h |
| | 评论系统 | P2 | 6h |
| **性能优化** | 虚拟滚动 | P0 | 8h |
| | 大文件优化 | P1 | 6h |
| | 缓存策略 | P1 | 4h |
| **错误处理** | 错误本地化 | P1 | 8h |
| | 错误恢复建议 | P1 | 6h |
| | 错误上报 | P2 | 4h |

---

## 🎯 核心改进方案

### 改进1：顶层项目管理器

#### 需求分析

**当前问题**:
- 程序启动时读取当前目录的 `.cowork`，无法跨目录管理项目
- 用户需要在不同的工作目录间切换，体验不流畅
- 缺少统一的项目列表视图和操作入口

**目标**:
- 在系统应用配置存储位置（如 Windows AppData）管理所有项目记录
- 主界面新增 "Projects" Tab，显示项目列表
- 支持 Open（新窗口打开）、Delete、Rename 等操作
- Open 操作在新进程/窗口中打开指定项目，避免对现有实现改动过大

#### 技术方案

**1. 项目注册表设计**

```rust
// cowork-gui/src-tauri/src/project_registry.rs

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// 项目注册信息
#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectRecord {
    /// 项目唯一ID
    pub project_id: String,
    /// 项目名称
    pub name: String,
    /// 项目描述
    pub description: Option<String>,
    /// 项目工作目录（绝对路径）
    pub workspace_path: PathBuf,
    /// config.toml 路径
    pub config_path: PathBuf,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后打开时间
    pub last_opened_at: DateTime<Utc>,
    /// 项目状态
    pub status: ProjectStatus,
    /// 项目类型（自动检测）
    pub project_type: String,
    /// 会话数量
    pub session_count: usize,
    /// 最后完成的会话ID
    pub last_completed_session: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum ProjectStatus {
    Active,      // 有进行中的会话
    Completed,   // 已完成，可继续开发
    Archived,    // 已归档
}

/// 项目注册表
#[derive(Serialize, Deserialize)]
pub struct ProjectRegistry {
    pub version: String,
    pub projects: Vec<ProjectRecord>,
    pub settings: RegistrySettings,
}

#[derive(Serialize, Deserialize)]
pub struct RegistrySettings {
    pub default_project_id: Option<String>,
    pub auto_create_workspace: bool,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            projects: Vec::new(),
            settings: RegistrySettings {
                default_project_id: None,
                auto_create_workspace: true,
            },
        }
    }
}
```

**2. 注册表存储位置**

```rust
// 获取应用数据目录
pub fn get_app_data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var("APPDATA")
            .or_else(|_| std::env::var("LOCALAPPDATA"))?;
        Ok(PathBuf::from(app_data).join("CoworkCreative"))
    }
    
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join("Library").join("Application Support").join("CoworkCreative"))
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join(".config").join("cowork-creative"))
    }
}

/// 注册表文件路径
pub fn get_registry_path() -> Result<PathBuf> {
    let app_data_dir = get_app_data_dir()?;
    Ok(app_data_dir.join("project_registry.json"))
}

/// 加载项目注册表
pub fn load_registry() -> Result<ProjectRegistry> {
    let registry_path = get_registry_path()?;
    
    if !registry_path.exists() {
        // 创建默认注册表
        let registry = ProjectRegistry::new();
        save_registry(&registry)?;
        return Ok(registry);
    }
    
    let content = std::fs::read_to_string(&registry_path)?;
    let registry: ProjectRegistry = serde_json::from_str(&content)?;
    Ok(registry)
}

/// 保存项目注册表
pub fn save_registry(registry: &ProjectRegistry) -> Result<()> {
    let registry_path = get_registry_path()?;
    
    // 确保目录存在
    if let Some(parent) = registry_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let content = serde_json::to_string_pretty(registry)?;
    std::fs::write(&registry_path, content)?;
    Ok(())
}
```

**3. 项目注册与发现**

```rust
/// 注册新项目
pub fn register_project(
    workspace_path: PathBuf,
    config_path: PathBuf,
    name: Option<String>,
) -> Result<ProjectRecord> {
    let mut registry = load_registry()?;
    
    // 生成项目ID
    let project_id = format!("proj-{}", chrono::Utc::now().timestamp_millis());
    
    // 自动检测项目名称
    let project_name = name.unwrap_or_else(|| {
        workspace_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled Project")
            .to_string()
    });
    
    // 检测项目类型
    let project_type = detect_project_type(&workspace_path);
    
    // 统计会话数量
    let session_count = count_sessions(&workspace_path)?;
    
    // 创建项目记录
    let record = ProjectRecord {
        project_id: project_id.clone(),
        name: project_name,
        description: None,
        workspace_path: workspace_path.clone(),
        config_path,
        created_at: chrono::Utc::now(),
        last_opened_at: chrono::Utc::now(),
        status: ProjectStatus::Active,
        project_type,
        session_count,
        last_completed_session: find_last_completed_session(&workspace_path)?,
    };
    
    // 检查是否已注册
    if registry.projects.iter().any(|p| p.workspace_path == workspace_path) {
        return Err(anyhow::anyhow!("Project already registered"));
    }
    
    registry.projects.push(record);
    save_registry(&registry)?;
    
    Ok(record)
}

/// 从当前目录自动注册项目
pub fn auto_register_current_project() -> Result<Option<ProjectRecord>> {
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("config.toml");
    
    // 检查是否是有效的 Cowork 项目
    if !config_path.exists() {
        return Ok(None);
    }
    
    let cowork_dir = current_dir.join(".cowork");
    if !cowork_dir.exists() {
        return Ok(None);
    }
    
    // 检查是否已注册
    let registry = load_registry()?;
    if registry.projects.iter().any(|p| p.workspace_path == current_dir) {
        return Ok(None);
    }
    
    // 自动注册
    let record = register_project(current_dir, config_path, None)?;
    Ok(Some(record))
}

/// 删除项目注册（仅删除记录，不删除实际项目文件）
pub fn unregister_project(project_id: &str) -> Result<()> {
    let mut registry = load_registry()?;
    
    registry.projects.retain(|p| p.project_id != project_id);
    save_registry(&registry)?;
    
    Ok(())
}

/// 更新项目最后打开时间
pub fn update_last_opened(project_id: &str) -> Result<()> {
    let mut registry = load_registry()?;
    
    if let Some(project) = registry.projects.iter_mut().find(|p| p.project_id == project_id) {
        project.last_opened_at = chrono::Utc::now();
        save_registry(&registry)?;
    }
    
    Ok(())
}
```

**4. Tauri 命令实现**

```rust
#[tauri::command]
pub async fn get_all_projects() -> Result<Vec<ProjectRecord>, String> {
    let registry = load_registry().map_err(|e| e.to_string())?;
    Ok(registry.projects)
}

#[tauri::command]
pub async fn get_project(project_id: String) -> Result<ProjectRecord, String> {
    let registry = load_registry().map_err(|e| e.to_string())?;
    
    registry
        .projects
        .into_iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())
}

#[tauri::command]
pub async fn register_project_from_path(
    workspace_path: String,
    name: Option<String>,
) -> Result<ProjectRecord, String> {
    let workspace = PathBuf::from(workspace_path);
    let config_path = workspace.join("config.toml");
    
    if !config_path.exists() {
        return Err("Invalid project directory: config.toml not found".to_string());
    }
    
    register_project(workspace, config_path, name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_project(project_id: String, delete_files: bool) -> Result<(), String> {
    let registry = load_registry().map_err(|e| e.to_string())?;
    
    let project = registry
        .projects
        .iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    // 如果需要删除文件
    if delete_files {
        std::fs::remove_dir_all(&project.workspace_path)
            .map_err(|e| format!("Failed to delete project files: {}", e))?;
    }
    
    // 删除注册记录
    unregister_project(&project_id).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn open_project_in_new_window(
    project_id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let registry = load_registry().map_err(|e| e.to_string())?;
    
    let project = registry
        .projects
        .iter()
        .find(|p| p.project_id == project_id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    // 更新最后打开时间
    update_last_opened(&project_id).map_err(|e| e.to_string())?;
    
    // 在新窗口中打开项目
    // 使用 Tauri 的 shell API 或启动新进程
    #[cfg(target_os = "windows")]
    {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        
        std::process::Command::new(exe_path)
            .arg("--workspace")
            .arg(&project.workspace_path)
            .spawn()
            .map_err(|e| format!("Failed to open new window: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn update_project_info(
    project_id: String,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), String> {
    let mut registry = load_registry().map_err(|e| e.to_string())?;
    
    if let Some(project) = registry.projects.iter_mut().find(|p| p.project_id == project_id) {
        if let Some(n) = name {
            project.name = n;
        }
        if let Some(d) = description {
            project.description = Some(d);
        }
        save_registry(&registry).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
```

**5. 前端 UI 实现**

```jsx
// ProjectsPanel.jsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { List, Button, Card, Tag, Space, Modal, Input, message } from 'antd';
import { FolderOpenOutlined, DeleteOutlined, EditOutlined, ClockOutlined } from '@ant-design/icons';

const ProjectsPanel = () => {
  const [projects, setProjects] = useState([]);
  const [loading, setLoading] = useState(true);
  const [deleteModalVisible, setDeleteModalVisible] = useState(false);
  const [selectedProject, setSelectedProject] = useState(null);
  const [editModalVisible, setEditModalVisible] = useState(false);
  const [editForm, setEditForm] = useState({ name: '', description: '' });

  const loadProjects = async () => {
    setLoading(true);
    try {
      const data = await invoke('get_all_projects');
      setProjects(data);
    } catch (error) {
      message.error('加载项目列表失败: ' + error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProjects();
  }, []);

  const handleOpenProject = async (project) => {
    try {
      await invoke('open_project_in_new_window', { projectId: project.project_id });
      message.success('正在打开项目...');
    } catch (error) {
      message.error('打开项目失败: ' + error);
    }
  };

  const handleDeleteProject = (project) => {
    setSelectedProject(project);
    setDeleteModalVisible(true);
  };

  const confirmDelete = async (deleteFiles) => {
    try {
      await invoke('delete_project', { 
        projectId: selectedProject.project_id, 
        deleteFiles 
      });
      message.success('项目已删除');
      setDeleteModalVisible(false);
      loadProjects();
    } catch (error) {
      message.error('删除项目失败: ' + error);
    }
  };

  const handleEditProject = (project) => {
    setSelectedProject(project);
    setEditForm({
      name: project.name,
      description: project.description || ''
    });
    setEditModalVisible(true);
  };

  const saveEdit = async () => {
    try {
      await invoke('update_project_info', {
        projectId: selectedProject.project_id,
        name: editForm.name,
        description: editForm.description
      });
      message.success('项目信息已更新');
      setEditModalVisible(false);
      loadProjects();
    } catch (error) {
      message.error('更新项目信息失败: ' + error);
    }
  };

  const formatDateTime = (timestamp) => {
    return new Date(timestamp).toLocaleString('zh-CN');
  };

  return (
    <div className="projects-panel">
      <div style={{ marginBottom: '20px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>📁 我的项目</h2>
        <Button type="primary" onClick={() => message.info('请使用菜单栏创建新项目')}>
          + 新建项目
        </Button>
      </div>

      {loading ? (
        <div>加载中...</div>
      ) : projects.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '60px 0', color: '#888' }}>
          <p>还没有项目</p>
          <p>创建你的第一个项目，或打开现有的 Cowork 项目目录</p>
        </div>
      ) : (
        <List
          grid={{ gutter: 16, xs: 1, sm: 2, md: 2, lg: 3, xl: 3, xxl: 4 }}
          dataSource={projects}
          renderItem={(project) => (
            <List.Item>
              <Card
                hoverable
                actions={[
                  <Button 
                    icon={<FolderOpenOutlined />} 
                    onClick={() => handleOpenProject(project)}
                    type="primary"
                  >
                    打开
                  </Button>,
                  <Button 
                    icon={<EditOutlined />} 
                    onClick={() => handleEditProject(project)}
                  >
                    编辑
                  </Button>,
                  <Button 
                    danger 
                    icon={<DeleteOutlined />} 
                    onClick={() => handleDeleteProject(project)}
                  >
                    删除
                  </Button>,
                ]}
              >
                <Card.Meta
                  title={project.name}
                  description={project.description || '暂无描述'}
                />
                <div style={{ marginTop: '12px' }}>
                  <Tag color={project.status === 'Active' ? 'green' : 'blue'}>
                    {project.status}
                  </Tag>
                  <Tag>{project.project_type}</Tag>
                </div>
                <div style={{ marginTop: '8px', fontSize: '12px', color: '#888' }}>
                  <div>📊 {project.session_count} 个会话</div>
                  <div><ClockOutlined /> 最后打开: {formatDateTime(project.last_opened_at)}</div>
                </div>
              </Card>
            </List.Item>
          )}
        />
      )}

      {/* 删除确认对话框 */}
      <Modal
        title="删除项目"
        open={deleteModalVisible}
        onOk={() => confirmDelete(false)}
        onCancel={() => setDeleteModalVisible(false)}
        okText="仅删除记录"
        okButtonProps={{ danger: true }}
        cancelText="取消"
      >
        <p>确定要删除项目 "{selectedProject?.name}" 吗？</p>
        <p style={{ color: '#ff4d4f', fontSize: '12px' }}>
          ⚠️ 此操作仅删除项目注册记录，不会删除实际的项目文件。
        </p>
      </Modal>

      {/* 编辑项目对话框 */}
      <Modal
        title="编辑项目"
        open={editModalVisible}
        onOk={saveEdit}
        onCancel={() => setEditModalVisible(false)}
      >
        <Space direction="vertical" style={{ width: '100%' }}>
          <div>
            <label>项目名称:</label>
            <Input 
              value={editForm.name} 
              onChange={(e) => setEditForm({...editForm, name: e.target.value})}
            />
          </div>
          <div>
            <label>项目描述:</label>
            <Input.TextArea 
              value={editForm.description}
              onChange={(e) => setEditForm({...editForm, description: e.target.value})}
              rows={3}
            />
          </div>
        </Space>
      </Modal>
    </div>
  );
};

export default ProjectsPanel;
```

**6. App.jsx 集成**

```jsx
// 在 App.jsx 中添加 Projects Tab
import ProjectsPanel from './components/ProjectsPanel';

// 在 Menu items 中添加
{ key: 'projects', icon: <FolderOutlined />, label: 'Projects' },

// 在 renderContent 中添加
case 'projects':
  return <ProjectsPanel />;
```

**7. 启动参数处理**

```rust
// main.rs
use tauri::utils::platform::current_platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    
    let workspace_path = if args.len() > 1 && args[1] == "--workspace" {
        Some(args.get(2).cloned())
    } else {
        None
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_project,
            // ... 其他命令
            get_all_projects,
            open_project_in_new_window,
            // ...
        ])
        .setup(move |app| {
            // 如果指定了工作区路径，切换到该目录
            if let Some(path) = workspace_path {
                std::env::set_current_dir(&path)
                    .expect("Failed to set working directory");
            }
            
            // 其他初始化...
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**预期效果**:
- ✅ 统一的项目管理界面
- ✅ 支持跨目录管理多个项目
- ✅ 新窗口打开项目，不影响当前工作
- ✅ 项目信息持久化存储
- ✅ 快速切换项目

**预计工时**: 40小时

---

### 改进2：智能迭代调度 Agent

#### 需求分析

**当前问题**:
- New 创建的项目在 Completed 后无法继续聊天交互
- 用户需要手动选择 Resume/Modify 操作，学习成本高
- 项目完成后继续开发的流程不够自然
- 缺少智能 Agent 来判断用户意图并自动调度

**目标**:
- 项目 Completed 后，聊天框继续可用
- 引入智能 Agent（IterativeAssistant）自动调度修改模式
- 根据用户输入智能判断是否需要 Resume/Modify/New
- 提供更自然的持续开发体验

#### 技术方案

**1. 迭代助手 Agent 设计**

```rust
// cowork-core/src/agents/iterative_assistant.rs

use anyhow::Result;
use adk_core::{Agent, InvocationContext, Event, EventStream};

/// 迭代助手 Agent - 负责智能调度修改模式
pub struct IterativeAssistant {
    llm: Arc<dyn Llm>,
}

impl IterativeAssistant {
    pub fn new(llm: Arc<dyn Llm>) -> Self {
        Self { llm }
    }
    
    /// 分析用户意图
    pub async fn analyze_user_intent(&self, 
        user_input: &str, 
        current_session_status: &SessionStatus,
        project_context: &ProjectContext
    ) -> Result<IterationIntent> {
        let prompt = format!(
            r#"你是一个迭代开发助手。请分析用户的输入，判断用户想要做什么。

当前项目状态：
- 项目名称: {}
- 会话状态: {:?}
- 最后完成阶段: {}
- 代码文件数量: {}

用户输入: {}

请判断用户的意图，返回以下选项之一：
1. CONTINUE_DEVELOPMENT - 继续开发当前项目（需要使用 Modify 模式）
2. START_NEW_FEATURE - 开发新功能（需要使用 Modify 模式）
3. FIX_BUG - 修复问题（需要使用 Modify 模式）
4. REFACTOR - 重构代码（需要使用 Modify 模式）
5. NEW_PROJECT - 创建全新项目
6. CLARIFICATION - 需要更多信息

返回格式：{{"intent": "INTENT_TYPE", "confidence": 0.9, "reasoning": "原因说明"}}"#,
            project_context.name,
            current_session_status,
            project_context.last_completed_stage,
            project_context.code_file_count,
            user_input
        );
        
        let response = self.llm.generate(&prompt).await?;
        let intent: IterationIntent = serde_json::from_str(&response)?;
        Ok(intent)
    }
    
    /// 生成修改建议
    pub async fn generate_modify_suggestion(&self,
        user_input: &str,
        project_context: &ProjectContext
    ) -> Result<ModifySuggestion> {
        let prompt = format!(
            r#"用户想要对现有项目进行修改。

项目信息：
- 项目名称: {}
- 项目描述: {}
- 已完成功能: {}
- 代码文件: {}

用户需求: {}

请生成修改建议，包括：
1. 修改类型（功能增强、Bug修复、重构等）
2. 受影响的模块
3. 建议的实施方案
4. 风险评估

返回格式：{{
    "modify_type": "类型",
    "affected_modules": ["模块1", "模块2"],
    "implementation_plan": "实施计划",
    "risk_assessment": "风险评估"
}}"#,
            project_context.name,
            project_context.description,
            project_context.completed_features.join(", "),
            project_context.code_files.join(", "),
            user_input
        );
        
        let response = self.llm.generate(&prompt).await?;
        let suggestion: ModifySuggestion = serde_json::from_str(&response)?;
        Ok(suggestion)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationIntent {
    pub intent: IntentType,
    pub confidence: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentType {
    ContinueDevelopment,
    StartNewFeature,
    FixBug,
    Refactor,
    NewProject,
    Clarification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifySuggestion {
    pub modify_type: String,
    pub affected_modules: Vec<String>,
    pub implementation_plan: String,
    pub risk_assessment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub description: String,
    pub last_completed_stage: String,
    pub code_file_count: usize,
    pub completed_features: Vec<String>,
    pub code_files: Vec<String>,
}
```

**2. 持续聊天交互增强**

```rust
// cowork-gui/src-tauri/src/lib.rs

#[tauri::command]
async fn send_chat_message(
    message: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Chat message: {}", message);
    
    // 检查当前是否有活跃会话
    let current_session = get_current_session()?;
    
    match current_session.status {
        SessionStatus::InProgress => {
            // 会话进行中，直接转发消息
            handle_active_session_message(message, window, state).await
        }
        SessionStatus::Completed => {
            // 会话已完成，需要智能调度
            handle_completed_session_message(message, window, state).await
        }
        SessionStatus::Failed => {
            // 会话失败，建议恢复
            Err("会话失败，请选择恢复或重新创建".to_string())
        }
    }
}

async fn handle_completed_session_message(
    message: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 加载项目上下文
    let project_context = load_project_context()?;
    
    // 创建迭代助手
    let llm = load_llm_config()?;
    let assistant = IterativeAssistant::new(llm);
    
    // 分析用户意图
    let intent = assistant.analyze_user_intent(&message, &SessionStatus::Completed, &project_context).await
        .map_err(|e| format!("分析意图失败: {}", e))?;
    
    info!("User intent: {:?} (confidence: {})", intent.intent, intent.confidence);
    
    match intent.intent {
        IntentType::ContinueDevelopment |
        IntentType::StartNewFeature |
        IntentType::FixBug |
        IntentType::Refactor => {
            // 自动启动 Modify 流程
            info!("Starting modify workflow for intent: {:?}", intent.intent);
            
            // 生成修改建议
            let suggestion = assistant.generate_modify_suggestion(&message, &project_context).await
                .map_err(|e| format!("生成修改建议失败: {}", e))?;
            
            // 发送建议到前端
            window.emit("modify_suggestion", serde_json::json!({
                "intent": intent,
                "suggestion": suggestion,
                "user_input": message
            }))?;
            
            // 显示确认对话框
            window.emit("confirm_modify", serde_json::json!({
                "message": format!("检测到您想要: {}", intent.reasoning),
                "suggestion": suggestion,
                "auto_mode": true
            }))?;
            
            Ok("waiting_for_confirmation".to_string())
        }
        IntentType::NewProject => {
            // 建议创建新项目
            window.emit("suggest_new_project", serde_json::json!({
                "message": "检测到您想要创建一个新项目",
                "user_input": message
            }))?;
            Ok("suggest_new_project".to_string())
        }
        IntentType::Clarification => {
            // 需要更多信息
            window.emit("request_clarification", serde_json::json!({
                "message": intent.reasoning,
                "user_input": message
            }))?;
            Ok("request_clarification".to_string())
        }
    }
}

#[tauri::command]
async fn confirm_auto_modify(
    user_input: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Auto-modify confirmed: {}", user_input);
    
    // 获取当前会话ID
    let current_session = get_current_session()?;
    
    // 创建 Modify 类型的会话
    let new_session_id = create_modify_session(&current_session.session_id, &user_input)?;
    
    // 执行 Modify Pipeline
    let result = execute_modify_pipeline(&new_session_id, window, state).await?;
    
    Ok(new_session_id)
}

#[tauri::command]
async fn cancel_auto_modify() -> Result<(), String> {
    info!("Auto-modify canceled");
    Ok(())
}
```

**3. 前端智能交互增强**

```jsx
// App.jsx - 增强聊天交互

useEffect(() => {
    // ... 现有的事件监听 ...
    
    // 监听修改建议
    const unlistenModifySuggestion = listen('modify_suggestion', (event) => {
        const { intent, suggestion, user_input } = event.payload;
        
        Modal.confirm({
            title: '继续开发',
            width: 600,
            content: (
                <div>
                    <p><strong>检测到您的意图:</strong></p>
                    <p>{intent.reasoning}</p>
                    
                    <Divider />
                    
                    <p><strong>修改建议:</strong></p>
                    <p><strong>类型:</strong> {suggestion.modify_type}</p>
                    <p><strong>受影响模块:</strong> {suggestion.affected_modules.join(', ')}</p>
                    <p><strong>实施计划:</strong></p>
                    <pre style={{ background: '#f5f5f5', padding: '10px', borderRadius: '4px' }}>
                        {suggestion.implementation_plan}
                    </pre>
                    <p><strong>风险评估:</strong> {suggestion.risk_assessment}</p>
                </div>
            ),
            okText: '开始修改',
            cancelText: '取消',
            onOk: async () => {
                try {
                    const newSessionId = await invoke('confirm_auto_modify', { 
                        userInput: user_input 
                    });
                    setCurrentSession(newSessionId);
                    setMessages(prev => [...prev, { 
                        type: 'system', 
                        content: '✅ 已启动修改流程，正在生成代码...', 
                        timestamp: new Date().toISOString() 
                    }]);
                } catch (error) {
                    message.error('启动修改失败: ' + error);
                }
            },
            onCancel: async () => {
                await invoke('cancel_auto_modify');
            }
        });
    });
    
    // 监听新项目建议
    const unlistenSuggestNewProject = listen('suggest_new_project', (event) => {
        const { message, user_input } = event.payload;
        
        Modal.confirm({
            title: '创建新项目',
            content: (
                <div>
                    <p>{message}</p>
                    <p>您的输入: "{user_input}"</p>
                    <p>是否创建一个新项目？</p>
                </div>
            ),
            okText: '创建新项目',
            cancelText: '取消',
            onOk: () => {
                setProjectIdea(user_input);
                handleCreateProject();
            }
        });
    });
    
    // 监听澄清请求
    const unlistenRequestClarification = listen('request_clarification', (event) => {
        const { message, user_input } = event.payload;
        
        message.warning(message);
        setMessages(prev => [...prev, { 
            type: 'system', 
            content: `❓ ${message}，请提供更多信息。`, 
            timestamp: new Date().toISOString() 
        }]);
    });
    
    return () => {
        unlistenModifySuggestion.then(f => f());
        unlistenSuggestNewProject.then(f => f());
        unlistenRequestClarification.then(f => f());
    };
}, []);

// 修改发送消息的逻辑
const handleSendUserMessage = async () => {
    if (!userInput.trim()) return;
    
    // 添加用户消息
    setMessages(prev => [...prev, { 
        type: 'user', 
        content: userInput, 
        timestamp: new Date().toISOString() 
    }]);
    
    const message = userInput.trim();
    setUserInput('');
    
    // 检查是否是 HITL 响应
    if (inputRequest) {
        await invoke('submit_input_response', { 
            requestId: inputRequest.requestId, 
            response: message, 
            responseType: 'text' 
        });
        setInputRequest(null);
    } else {
        // 普通聊天消息
        try {
            const result = await invoke('send_chat_message', { message });
            
            // 如果需要确认，等待用户操作
            if (result !== 'waiting_for_confirmation' && result !== 'suggest_new_project') {
                setMessages(prev => [...prev, { 
                    type: 'system', 
                    content: '💭 正在处理您的请求...', 
                    timestamp: new Date().toISOString() 
                }]);
            }
        } catch (error) {
            message.error('发送消息失败: ' + error);
        }
    }
};
```

**4. 项目上下文加载**

```rust
fn load_project_context() -> Result<ProjectContext> {
    let index = load_project_index()?;
    
    // 获取项目信息
    let project_name = index.project_name.clone();
    let project_description = index.description.clone();
    
    // 获取最新的已完成会话
    let last_completed = index.sessions
        .iter()
        .filter(|s| s.status == SessionStatus::Completed)
        .max_by_key(|s| s.created_at);
    
    let last_completed_stage = last_completed
        .and_then(|s| s.last_completed_stage.clone())
        .unwrap_or_else(|| "unknown".to_string());
    
    // 统计代码文件
    let code_files = collect_code_files()?;
    let code_file_count = code_files.len();
    
    // 获取已完成功能
    let completed_features = load_completed_features()?;
    
    Ok(ProjectContext {
        name: project_name,
        description: project_description,
        last_completed_stage,
        code_file_count,
        completed_features,
        code_files,
    })
}

fn collect_code_files() -> Result<Vec<String>> {
    let mut files = Vec::new();
    let current_dir = std::env::current_dir()?;
    
    // 遍历项目目录，收集代码文件
    for entry in walkdir::WalkDir::new(&current_dir)
        .into_iter()
        .filter_entry(|e| !e.path().starts_with(".cowork") && !e.path().starts_with("target"))
    {
        let entry = entry?;
        if entry.path().is_file() {
            if let Some(ext) = entry.path().extension() {
                if matches!(ext.to_str(), Some("rs") | Some("js") | Some("jsx") | Some("ts") | Some("tsx") | Some("py")) {
                    if let Ok(path) = entry.path().strip_prefix(&current_dir) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    Ok(files)
}

fn load_completed_features() -> Result<Vec<String>> {
    let index = load_project_index()?;
    
    // 从所有已完成会话中提取功能
    let mut features = Vec::new();
    
    for session in &index.sessions {
        if session.status == SessionStatus::Completed {
            if let Ok(session_dir) = get_session_dir(&session.session_id) {
                let features_file = session_dir.join("artifacts").join("features.json");
                if features_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(features_file) {
                        if let Ok(feature_list) = serde_json::from_str::<FeatureList>(&content) {
                            features.extend(feature_list.features.iter().map(|f| f.name.clone()));
                        }
                    }
                }
            }
        }
    }
    
    Ok(features)
}
```

**5. 创建 Modify 会话**

```rust
fn create_modify_session(base_session_id: &str, user_input: &str) -> Result<String> {
    let new_session_id = format!("session-{}", chrono::Utc::now().timestamp());
    
    let mut index = load_project_index()?;
    
    // 创建 Modify 类型的会话记录
    let session_record = SessionRecord {
        session_id: new_session_id.clone(),
        session_type: SessionType::Modify,
        created_at: chrono::Utc::now(),
        completed_at: None,
        status: SessionStatus::InProgress,
        base_session_id: Some(base_session_id.to_string()),
        input_description: user_input.to_string(),
        change_request_id: None,
    };
    
    index.add_session(session_record);
    save_project_index(&index)?;
    
    // 保存会话输入
    let session_input = SessionInput {
        session_id: new_session_id.clone(),
        session_type: SessionType::Modify,
        description: user_input.to_string(),
        base_session_id: Some(base_session_id.to_string()),
        created_at: chrono::Utc::now(),
    };
    
    save_session_input(&new_session_id, &session_input)?;
    
    Ok(new_session_id)
}

async fn execute_modify_pipeline(
    session_id: &str,
    window: Window,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 加载配置
    let config = load_config()?;
    
    // 获取基础会话
    let index = load_project_index()?;
    let base_session = index.sessions
        .iter()
        .find(|s| s.session_id == session_id)
        .ok_or("Session not found")?;
    
    let base_session_id = base_session.base_session_id
        .as_ref()
        .ok_or("No base session")?;
    
    // 创建交互后端
    let interaction = Arc::new(TauriBackend::new(
        window.app_handle().clone(),
        state.event_bus.clone(),
        state.pending_requests.clone(),
    ));
    
    // 创建 Modify Pipeline
    let pipeline = create_modify_pipeline(&config, session_id, base_session_id, interaction)
        .map_err(|e| format!("Failed to create modify pipeline: {}", e))?;
    
    // 在后台执行
    let pipeline_clone = pipeline;
    let session_id_clone = session_id.to_string();
    let window_clone = window.app_handle().clone();
    
    tokio::spawn(async move {
        // ... 执行 pipeline 的逻辑 ...
        let result = execute_pipeline_async(pipeline_clone, session_id_clone, window_clone).await;
        
        match result {
            Ok(_) => {
                mark_session_completed(session_id).ok();
            }
            Err(e) => {
                mark_session_failed(session_id, &e.to_string()).ok();
            }
        }
    });
    
    Ok(session_id.to_string())
}
```

**6. 用户界面优化**

```jsx
// 在聊天界面添加状态指示器
const renderChatHeader = () => {
    const currentSessionData = sessions.find(s => s.id === currentSession);
    
    return (
        <div style={{ 
            padding: '12px 20px', 
            borderBottom: '1px solid #303030',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center'
        }}>
            <div>
                <h3 style={{ margin: 0 }}>💬 对话</h3>
                {currentSessionData && (
                    <span style={{ fontSize: '12px', color: '#888' }}>
                        会话: {currentSessionData.id.substring(0, 12)}... · 
                        状态: {currentSessionData.status}
                    </span>
                )}
            </div>
            {currentSessionData?.status === 'Completed' && (
                <Tag color="blue">✨ 可继续开发 - 直接发送消息即可</Tag>
            )}
        </div>
    );
};
```

**预期效果**:
- ✅ 项目完成后继续交互
- ✅ 智能判断用户意图
- ✅ 自动调度 Modify 模式
- ✅ 提供修改建议和风险评估
- ✅ 更自然的持续开发体验

**预计工时**: 32小时

---

## 🚀 迭代路线图（更新版）

### Phase 1: 核心体验优化 (2-3周)

**目标**: 修复关键问题，提升基础体验

#### 1.1 实现实时日志流 (P0)
**问题描述**: RunnerPanel 只显示启动时的静态日志，无法实时查看项目运行输出

**解决方案**:
```rust
// 在 project_runner.rs 中添加实时日志推送
pub async fn start(&self, session_id: String, command: String, window: Window) -> Result<u32, String> {
    // ... 启动进程 ...
    
    // 实时推送 stdout
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        while reader.read_line(&mut line).await? > 0 {
            window.emit("log_output", (session_id.clone(), "stdout", line.clone()))?;
            line.clear();
        }
        Ok::<(), anyhow::Error>(())
    });
    
    // 实时推送 stderr
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        while reader.read_line(&mut line).await? > 0 {
            window.emit("log_output", (session_id.clone(), "stderr", line.clone()))?;
            line.clear();
        }
        Ok::<(), anyhow::Error>(())
    });
}
```

**前端实现**:
```jsx
// RunnerPanel.jsx
useEffect(() => {
  const unlisten = listen('log_output', (event) => {
    const [sessionId, stream, line] = event.payload;
    if (sessionId === currentSession) {
      setLogs(prev => [...prev, { stream, content: line, timestamp: Date.now() }]);
    }
  });
  return () => unlisten.then(f => f());
}, [currentSession]);
```

**预期效果**:
- ✅ 实时显示项目运行日志
- ✅ 区分 stdout 和 stderr
- ✅ 日志自动滚动到底部
- ✅ 支持日志复制和清空

**预计工时**: 8小时

#### 1.2 性能优化：虚拟滚动 (P0)
**问题描述**: 文件树在大型项目中性能差，卡顿严重

**解决方案**:
```jsx
// 使用 react-window 实现虚拟滚动
import { FixedSizeTree as Tree } from 'react-vtree';

const CodeEditor = () => {
  return (
    <Tree
      treeWalker={treeWalker}
      height={600}
      width={300}
      itemSize={30}
      renderItem={({ data, depth, isOpen, style }) => (
        <div style={style}>
          {/* 渲染文件节点 */}
        </div>
      )}
    />
  );
};
```

**预期效果**:
- ✅ 支持数千个文件流畅渲染
- ✅ 降低内存占用
- ✅ 提升首屏加载速度

**预计工时**: 8小时

#### 1.3 错误提示优化 (P1)
**问题描述**: Rust 错误信息直接显示，用户难以理解

**解决方案**:
```rust
// 创建错误映射表
const ERROR_MESSAGES: &[(ErrorCode, UserFriendlyMessage)] = &[
    (ErrorCode::FileNotFound, "文件未找到，请检查路径是否正确"),
    (ErrorCode::PermissionDenied, "权限不足，请检查文件权限"),
    // ...
];

pub fn format_error(error: &CoworkError) -> String {
    ERROR_MESSAGES.iter()
        .find(|(code, _)| error.code() == *code)
        .map(|(_, msg)| msg.to_string())
        .unwrap_or_else(|| error.to_string())
}
```

**预期效果**:
- ✅ 用户友好的错误提示
- ✅ 提供错误恢复建议
- ✅ 支持多语言（未来）

**预计工时**: 8小时

#### 1.4 文件搜索功能 (P1)
**问题描述**: 无法在项目中搜索文件或代码

**解决方案**:
```jsx
// 添加搜索框和快捷键 Ctrl+P
const CodeEditor = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState([]);

  const handleSearch = async (query) => {
    const results = await invoke('search_files', { query });
    setSearchResults(results);
  };

  return (
    <Modal open={showSearch}>
      <Input
        value={searchQuery}
        onChange={(e) => handleSearch(e.target.value)}
        placeholder="搜索文件 (Ctrl+P)"
      />
      <List
        dataSource={searchResults}
        renderItem={(file) => (
          <List.Item onClick={() => openFile(file.path)}>
            {file.path} - {file.preview}
          </List.Item>
        )}
      />
    </Modal>
  );
};
```

**预期效果**:
- ✅ 快速文件搜索 (Ctrl+P)
- ✅ 代码内容搜索
- ✅ 支持正则表达式
- ✅ 显示匹配行号

**预计工时**: 6小时

**Phase 1 总计**: 30小时 (约4个工作日)

---

### Phase 2: 专业创作环境 (3-4周)

**目标**: 提供类似专业 IDE 的开发体验

#### 2.1 代码格式化集成 (P1)
**功能描述**: 支持代码自动格式化，保持代码风格一致

**技术方案**:
```rust
// 集成 Prettier (JavaScript/TypeScript)
#[tauri::command]
pub async fn format_file(file_path: String, language: String) -> Result<String, String> {
    let content = fs::read_to_string(&file_path)?;
    
    match language.as_str() {
        "javascript" | "typescript" => {
            // 调用 Prettier CLI
            let output = Command::new("prettier")
                .arg("--stdin-filepath")
                .arg(&file_path)
                .pipe(stdin)
                .output()?;
            Ok(String::from_utf8(output.stdout)?)
        },
        "rust" => {
            // 调用 rustfmt
            let output = Command::new("rustfmt")
                .stdin(Stdio::piped())
                .pipe(stdin)
                .output()?;
            Ok(String::from_utf8(output.stdout)?)
        },
        _ => Ok(content),
    }
}
```

**预期效果**:
- ✅ 支持 JavaScript/TypeScript 格式化
- ✅ 支持 Rust 格式化
- ✅ 支持保存时自动格式化
- ✅ 自定义格式化规则

**预计工时**: 6小时

#### 2.2 代码导航功能 (P2)
**功能描述**: 支持定义跳转、引用查找、符号搜索

**技术方案**:
```rust
// 集成 rust-analyzer (Rust)
#[tauri::command]
pub async fn goto_definition(file_path: String, line: u32, column: u32) -> Result<Position, String> {
    let output = Command::new("rust-analyzer")
        .arg("gotoDefinition")
        .arg(&file_path)
        .arg(line.to_string())
        .arg(column.to_string())
        .output()?;
    
    let position: Position = serde_json::from_slice(&output.stdout)?;
    Ok(position)
}
```

**预期效果**:
- ✅ F12 跳转到定义
- ✅ Shift+F12 查找引用
- ✅ Ctrl+Shift+O 符号搜索
- ✅ 支持 Rust 和 JavaScript/TypeScript

**预计工时**: 8小时

#### 2.3 Git 集成 (P2)
**功能描述**: 查看 Git Diff、提交代码、版本历史

**技术方案**:
```rust
#[tauri::command]
pub async fn get_git_diff(file_path: String) -> Result<String, String> {
    let output = Command::new("git")
        .arg("diff")
        .arg(&file_path)
        .output()?;
    
    Ok(String::from_utf8(output.stdout)?)
}

#[tauri::command]
pub async fn git_commit(message: String) -> Result<(), String> {
    Command::new("git")
        .arg("add")
        .arg(".")
        .output()?;
    
    Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&message)
        .output()?;
    
    Ok(())
}
```

**UI 设计**:
```jsx
const GitPanel = () => {
  const [changes, setChanges] = useState([]);

  return (
    <div className="git-panel">
      <h3>Git Changes</h3>
      <List
        dataSource={changes}
        renderItem={(file) => (
          <List.Item>
            <div className="file-path">{file.path}</div>
            <DiffViewer oldText={file.oldContent} newText={file.newContent} />
          </List.Item>
        )}
      />
      <Button onClick={handleCommit}>Commit Changes</Button>
    </div>
  );
};
```

**预期效果**:
- ✅ 查看 Git Diff
- ✅ 提交代码
- ✅ 查看版本历史
- ✅ 撤销更改

**预计工时**: 12小时

#### 2.4 项目模板系统 (P1)
**功能描述**: 提供预定义的项目模板，快速启动项目

**技术方案**:
```toml
# 模板配置
[[templates]]
name = "React + Vite"
description = "现代化的 React 开发环境"
language = "javascript"
files = [
    "src/App.jsx",
    "src/main.jsx",
    "package.json",
    "vite.config.js"
]

[[templates]]
name = "Rust CLI"
description = "命令行工具模板"
language = "rust"
files = [
    "src/main.rs",
    "Cargo.toml",
]
```

```rust
#[tauri::command]
pub async fn list_templates() -> Result<Vec<Template>, String> {
    let config = load_template_config()?;
    Ok(config.templates)
}

#[tauri::command]
pub async fn apply_template(template_name: String, session_id: String) -> Result<(), String> {
    let template = find_template(&template_name)?;
    
    // 复制模板文件到项目目录
    for file in &template.files {
        let src = format!("templates/{}/{}", template_name, file);
        let dst = format!("sessions/{}/code/{}", session_id, file);
        fs::copy(src, dst)?;
    }
    
    Ok(())
}
```

**预期效果**:
- ✅ 提供多种项目模板
- ✅ 支持自定义模板
- ✅ 快速启动新项目
- ✅ 模板市场（未来）

**预计工时**: 12小时

**Phase 2 总计**: 38小时 (约5个工作日)

---

### Phase 3: 创作流程可视化 (2-3周)

**目标**: 让用户清晰了解 AI 智能体的工作过程和决策依据

#### 3.1 AI 思考过程可视化 (P1)
**功能描述**: 实时显示 AI 智能体的思考过程和决策依据

**技术方案**:
```jsx
const AgentThinkingViewer = () => {
  const [thinkingLogs, setThinkingLogs] = useState([]);

  useEffect(() => {
    const unlisten = listen('agent_thinking', (event) => {
      const { agent, stage, content, confidence } = event.payload;
      setThinkingLogs(prev => [...prev, {
        agent,
        stage,
        content,
        confidence,
        timestamp: Date.now()
      }]);
    });
    return () => unlisten.then(f => f());
  }, []);

  return (
    <div className="thinking-viewer">
      <h3>AI 思考过程</h3>
      <Timeline>
        {thinkingLogs.map((log, idx) => (
          <Timeline.Item key={idx}>
            <div className="agent-name">{log.agent}</div>
            <div className="stage">{log.stage}</div>
            <div className="content">{log.content}</div>
            <div className="confidence">
              置信度: {(log.confidence * 100).toFixed(0)}%
            </div>
          </Timeline.Item>
        ))}
      </Timeline>
    </div>
  );
};
```

**预期效果**:
- ✅ 实时显示 AI 思考过程
- ✅ 显示决策置信度
- ✅ 支持展开/折叠详细信息
- ✅ 时间线视图

**预计工时**: 10小时

#### 3.2 阶段进度可视化 (P1)
**功能描述**: 可视化显示当前项目所处的阶段和进度

**技术方案**:
```jsx
const StageProgress = ({ currentStage, stages }) => {
  return (
    <div className="stage-progress">
      <Steps current={currentStage}>
        {stages.map((stage, idx) => (
          <Step
            key={idx}
            title={stage.name}
            description={stage.description}
            status={getStageStatus(stage)}
          />
        ))}
      </Steps>
      <Progress percent={calculateProgress()} />
    </div>
  );
};
```

**预期效果**:
- ✅ 显示当前项目所处阶段
- ✅ 显示各阶段完成状态
- ✅ 显示整体进度百分比
- ✅ 支持点击跳转到指定阶段

**预计工时**: 6小时

#### 3.3 决策历史记录 (P2)
**功能描述**: 记录所有关键决策点，支持回溯和审计

**技术方案**:
```rust
#[derive(Serialize, Deserialize)]
pub struct DecisionRecord {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub stage: String,
    pub agent: String,
    pub decision_type: DecisionType,
    pub input: String,
    pub output: String,
    pub rationale: String,
    pub user_feedback: Option<String>,
}

#[tauri::command]
pub async fn get_decision_history(session_id: String) -> Result<Vec<DecisionRecord>, String> {
    let session_dir = get_session_dir(&session_id)?;
    let history_file = session_dir.join("decision_history.json");
    
    if history_file.exists() {
        let content = fs::read_to_string(history_file)?;
        let history: Vec<DecisionRecord> = serde_json::from_str(&content)?;
        Ok(history)
    } else {
        Ok(vec![])
    }
}
```

**UI 设计**:
```jsx
const DecisionHistory = ({ sessionId }) => {
  const [history, setHistory] = useState([]);

  useEffect(() => {
    invoke('get_decision_history', { sessionId }).then(setHistory);
  }, [sessionId]);

  return (
    <div className="decision-history">
      <h3>决策历史</h3>
      <Timeline>
        {history.map((record, idx) => (
          <Timeline.Item key={idx}>
            <div className="timestamp">{record.timestamp}</div>
            <div className="stage">{record.stage} - {record.agent}</div>
            <div className="decision-type">{record.decision_type}</div>
            <div className="input">输入: {record.input}</div>
            <div className="output">输出: {record.output}</div>
            <div className="rationale">依据: {record.rationale}</div>
            {record.user_feedback && (
              <div className="feedback">用户反馈: {record.user_feedback}</div>
            )}
          </Timeline.Item>
        ))}
      </Timeline>
    </div>
  );
};
```

**预期效果**:
- ✅ 记录所有关键决策
- ✅ 显示决策依据
- ✅ 显示用户反馈
- ✅ 支持导出决策历史

**预计工时**: 8小时

**Phase 3 总计**: 24小时 (约3个工作日)

---

### Phase 4: 多项目管理 (2-3周)

**目标**: 支持多项目并行管理，满足多场景开发需求

#### 4.1 项目工作区 (P1)
**功能描述**: 支持多项目工作区，快速切换项目

**技术方案**:
```jsx
const Workspace = () => {
  const [projects, setProjects] = useState([]);
  const [activeProject, setActiveProject] = useState(null);

  return (
    <div className="workspace">
      <div className="project-selector">
        <Select
          value={activeProject}
          onChange={setActiveProject}
          options={projects.map(p => ({ label: p.name, value: p.id }))}
        />
        <Button onClick={handleCreateProject}>新建项目</Button>
      </div>
      <div className="project-content">
        {activeProject && <ProjectView projectId={activeProject} />}
      </div>
    </div>
  );
};
```

**预期效果**:
- ✅ 支持多项目工作区
- ✅ 快速切换项目
- ✅ 项目状态独立
- ✅ 支持项目标签和分组

**预计工时**: 8小时

#### 4.2 项目导入/导出 (P2)
**功能描述**: 支持导入现有项目，导出项目配置

**技术方案**:
```rust
#[tauri::command]
pub async fn export_project(session_id: String, export_path: String) -> Result<(), String> {
    // 打包项目文件
    let session_dir = get_session_dir(&session_id)?;
    let archive = File::create(&export_path)?;
    let mut zip = ZipWriter::new(archive);
    
    // 添加所有文件到 zip
    for entry in WalkDir::new(&session_dir) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(&session_dir)?;
        
        if path.is_file() {
            zip.start_file(relative.to_str().unwrap(), FileOptions::default())?;
            let mut file = File::open(path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }
    
    zip.finish()?;
    Ok(())
}

#[tauri::command]
pub async fn import_project(archive_path: String) -> Result<String, String> {
    // 解压项目文件
    let file = File::open(&archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    let session_id = format!("session-{}", chrono::Utc::now().timestamp());
    let session_dir = get_session_dir(&session_id)?;
    fs::create_dir_all(&session_dir)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = session_dir.join(file.name());
        if file.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }
    
    Ok(session_id)
}
```

**预期效果**:
- ✅ 导出项目为 zip 文件
- ✅ 导入项目
- ✅ 保留所有配置和历史
- ✅ 支持项目分享

**预计工时**: 10小时

#### 4.3 项目配置管理 (P2)
**功能描述**: 集中管理项目配置，支持环境变量、构建脚本等

**技术方案**:
```rust
#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub description: String,
    pub language: String,
    pub build_command: String,
    pub test_command: String,
    pub environment_variables: HashMap<String, String>,
    pub dependencies: Vec<String>,
}

#[tauri::command]
pub async fn get_project_config(session_id: String) -> Result<ProjectConfig, String> {
    let config_path = get_session_dir(&session_id)?.join("project_config.json");
    let content = fs::read_to_string(config_path)?;
    let config: ProjectConfig = serde_json::from_str(&content)?;
    Ok(config)
}

#[tauri::command]
pub async fn save_project_config(session_id: String, config: ProjectConfig) -> Result<(), String> {
    let config_path = get_session_dir(&session_id)?.join("project_config.json");
    let content = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, content)?;
    Ok(())
}
```

**UI 设计**:
```jsx
const ProjectConfigPanel = ({ sessionId }) => {
  const [config, setConfig] = useState(null);

  useEffect(() => {
    invoke('get_project_config', { sessionId }).then(setConfig);
  }, [sessionId]);

  return (
    <div className="config-panel">
      <h3>项目配置</h3>
      <Form
        layout="vertical"
        initialValues={config}
        onFinish={handleSaveConfig}
      >
        <Form.Item name="name" label="项目名称">
          <Input />
        </Form.Item>
        <Form.Item name="description" label="项目描述">
          <Input.TextArea />
        </Form.Item>
        <Form.Item name="build_command" label="构建命令">
          <Input />
        </Form.Item>
        <Form.Item name="test_command" label="测试命令">
          <Input />
        </Form.Item>
        <Form.Item name="environment_variables" label="环境变量">
          <Input.TextArea placeholder='{"KEY": "VALUE"}' />
        </Form.Item>
        <Button type="primary" htmlType="submit">保存配置</Button>
      </Form>
    </div>
  );
};
```

**预期效果**:
- ✅ 集中管理项目配置
- ✅ 支持环境变量
- ✅ 支持自定义构建和测试命令
- ✅ 配置版本管理

**预计工时**: 8小时

**Phase 4 总计**: 26小时 (约3个工作日)

---

### Phase 5: 高级功能 (3-4周)

**目标**: 提供高级功能，满足专业用户需求

#### 5.1 插件系统 (P2)
**功能描述**: 支持自定义插件，扩展功能

**技术方案**:
```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_init(&self, context: &PluginContext) -> Result<(), Error>;
    fn on_event(&self, event: &Event, context: &PluginContext) -> Result<(), Error>;
    fn on_destroy(&self, context: &PluginContext) -> Result<(), Error>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, path: &Path) -> Result<(), Error> {
        // 加载动态库
        let lib = unsafe { libloading::Library::new(path)? };
        
        // 加载插件符号
        let create_plugin: libloading::Symbol<unsafe fn() -> *mut dyn Plugin> =
            unsafe { lib.get(b"create_plugin")? };
        
        let plugin = unsafe { Box::from_raw(create_plugin()) };
        self.plugins.push(plugin);
        
        Ok(())
    }
}
```

**插件示例**:
```rust
// 自定义代码格式化插件
pub struct CustomFormatterPlugin;

impl Plugin for CustomFormatterPlugin {
    fn name(&self) -> &str {
        "custom_formatter"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn on_event(&self, event: &Event, context: &PluginContext) -> Result<(), Error> {
        match event {
            Event::FileSaved { path, content } => {
                // 自定义格式化逻辑
                let formatted = self.format(content);
                context.save_file(path, formatted)?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

**预期效果**:
- ✅ 支持动态加载插件
- ✅ 插件生命周期管理
- ✅ 插件市场（未来）
- ✅ 插件开发者文档

**预计工时**: 20小时

#### 5.2 多主题支持 (P2)
**功能描述**: 支持多种主题，满足用户个性化需求

**技术方案**:
```jsx
// 使用 CSS Variables 实现主题切换
const themes = {
  dark: {
    '--bg-color': '#1f1f1f',
    '--text-color': '#ffffff',
    '--primary-color': '#1890ff',
    '--border-color': '#303030',
  },
  light: {
    '--bg-color': '#ffffff',
    '--text-color': '#000000',
    '--primary-color': '#1890ff',
    '--border-color': '#e0e0e0',
  },
  monokai: {
    '--bg-color': '#272822',
    '--text-color': '#f8f8f2',
    '--primary-color': '#a6e22e',
    '--border-color': '#3e3d32',
  },
};

const ThemeProvider = ({ children }) => {
  const [theme, setTheme] = useState('dark');
  
  return (
    <div style={themes[theme]}>
      {children}
      <Select value={theme} onChange={setTheme}>
        <Select.Option value="dark">Dark</Select.Option>
        <Select.Option value="light">Light</Select.Option>
        <Select.Option value="monokai">Monokai</Select.Option>
      </Select>
    </div>
  );
};
```

**预期效果**:
- ✅ 支持多种内置主题
- ✅ 支持自定义主题
- ✅ 主题实时切换
- ✅ 主题持久化

**预计工时**: 6小时

#### 5.3 快捷键系统 (P2)
**功能描述**: 支持自定义快捷键，提高操作效率

**技术方案**:
```jsx
import { HotKeys } from 'react-hotkeys';

const keyMap = {
  SAVE_FILE: 'ctrl+s',
  OPEN_FILE: 'ctrl+p',
  GOTO_DEFINITION: 'f12',
  FIND_REFERENCES: 'shift+f12',
  COMMAND_PALETTE: 'ctrl+shift+p',
};

const handlers = {
  SAVE_FILE: (event) => {
    event.preventDefault();
    saveFile();
  },
  OPEN_FILE: (event) => {
    event.preventDefault();
    showFileSearch();
  },
  GOTO_DEFINITION: (event) => {
    event.preventDefault();
    gotoDefinition();
  },
  FIND_REFERENCES: (event) => {
    event.preventDefault();
    findReferences();
  },
  COMMAND_PALETTE: (event) => {
    event.preventDefault();
    showCommandPalette();
  },
};

const App = () => {
  return (
    <HotKeys keyMap={keyMap} handlers={handlers}>
      <div className="app">
        {/* 应用内容 */}
      </div>
    </HotKeys>
  );
};
```

**预期效果**:
- ✅ 支持常用快捷键
- ✅ 支持自定义快捷键
- ✅ 快捷键提示
- ✅ 快捷键冲突检测

**预计工时**: 8小时

#### 5.4 命令面板 (P1)
**功能描述**: 提供统一的命令面板，快速执行各种操作

**技术方案**:
```jsx
const CommandPalette = () => {
  const [visible, setVisible] = useState(false);
  const [query, setQuery] = useState('');
  
  const commands = [
    { id: 'create-project', label: '创建新项目', action: () => createProject() },
    { id: 'open-settings', label: '打开设置', action: () => openSettings() },
    { id: 'toggle-sidebar', label: '切换侧边栏', action: () => toggleSidebar() },
    { id: 'run-project', label: '运行项目', action: () => runProject() },
    { id: 'build-project', label: '构建项目', action: () => buildProject() },
    // ...
  ];

  const filteredCommands = commands.filter(cmd =>
    cmd.label.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <Modal
      visible={visible}
      onCancel={() => setVisible(false)}
      footer={null}
      title="命令面板"
    >
      <Input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="输入命令 (Ctrl+Shift+P)"
        autoFocus
      />
      <List
        dataSource={filteredCommands}
        renderItem={(cmd) => (
          <List.Item onClick={() => { cmd.action(); setVisible(false); }}>
            {cmd.label}
          </List.Item>
        )}
      />
    </Modal>
  );
};
```

**预期效果**:
- ✅ 统一的命令面板
- ✅ 快速搜索和执行命令
- ✅ 支持命令分类
- ✅ 支持自定义命令

**预计工时**: 6小时

**Phase 5 总计**: 40小时 (约5个工作日)

---

## 📅 实施时间表

### 总体规划

| 阶段 | 目标 | 工作量 | 开始时间 | 结束时间 |
|------|------|--------|---------|---------|
| **Phase 1** | 核心体验优化 | 30h | Week 1 | Week 1.5 |
| **Phase 2** | 专业创作环境 | 38h | Week 2 | Week 3 |
| **Phase 3** | 创作流程可视化 | 24h | Week 4 | Week 4.5 |
| **Phase 4** | 多项目管理 | 26h | Week 5 | Week 5.5 |
| **Phase 5** | 高级功能 | 40h | Week 6 | Week 7.5 |
| **总计** | **全面升级** | **158h** | **Week 1** | **Week 8** |

### 甘特图

```
Week 1   Week 2   Week 3   Week 4   Week 5   Week 6   Week 7   Week 8
|--------|--------|--------|--------|--------|--------|--------|--------|
Phase 1: ████████
         Phase 2:         ██████████████
                          Phase 3:               ██████████
                                    Phase 4:               ██████████
                                               Phase 5:                      ████████████████
```

### 里程碑

| 里程碑 | 描述 | 时间点 | 验收标准 |
|--------|------|--------|---------|
| **M1** | 核心体验优化完成 | Week 1.5 | 实时日志流、虚拟滚动、错误提示优化、文件搜索 |
| **M2** | 专业创作环境完成 | Week 3 | 代码格式化、代码导航、Git 集成、项目模板 |
| **M3** | 创作流程可视化完成 | Week 4.5 | AI 思考过程可视化、阶段进度可视化、决策历史记录 |
| **M4** | 多项目管理完成 | Week 5.5 | 项目工作区、项目导入/导出、项目配置管理 |
| **M5** | 高级功能完成 | Week 7.5 | 插件系统、多主题支持、快捷键系统、命令面板 |

---

## 🎨 UI/UX 设计规范

### 设计原则

1. **简约优先**: 界面简洁，避免过度设计
2. **一致性**: 保持视觉和交互的一致性
3. **可访问性**: 支持键盘导航，满足无障碍需求
4. **响应式**: 适配不同屏幕尺寸
5. **性能优先**: 流畅的动画和交互

### 颜色规范

```css
/* Dark Theme (默认) */
--bg-primary: #1f1f1f;
--bg-secondary: #2a2a2a;
--bg-tertiary: #3a3a3a;
--text-primary: #ffffff;
--text-secondary: #b0b0b0;
--border-color: #404040;
--accent-color: #1890ff;
--success-color: #52c41a;
--warning-color: #faad14;
--error-color: #ff4d4f;

/* Light Theme */
--bg-primary: #ffffff;
--bg-secondary: #f5f5f5;
--bg-tertiary: #e8e8e8;
--text-primary: #000000;
--text-secondary: #666666;
--border-color: #d9d9d9;
```

### 字体规范

```css
/* 标题 */
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
font-weight: 600;
font-size: 18px;

/* 正文 */
font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
font-weight: 400;
font-size: 14px;

/* 代码 */
font-family: 'Fira Code', 'JetBrains Mono', 'Consolas', monospace;
font-size: 13px;
```

### 间距规范

```css
--spacing-xs: 4px;
--spacing-sm: 8px;
--spacing-md: 16px;
--spacing-lg: 24px;
--spacing-xl: 32px;
--spacing-xxl: 48px;
```

### 圆角规范

```css
--radius-sm: 4px;
--radius-md: 8px;
--radius-lg: 12px;
--radius-xl: 16px;
```

### 阴影规范

```css
--shadow-sm: 0 2px 4px rgba(0, 0, 0, 0.1);
--shadow-md: 0 4px 8px rgba(0, 0, 0, 0.15);
--shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.2);
```

---

## 🧪 测试策略

### 测试覆盖目标

| 测试类型 | 覆盖率目标 | 说明 |
|---------|-----------|------|
| **单元测试** | 70% | 核心逻辑和工具函数 |
| **集成测试** | 50% | 组件间交互 |
| **E2E 测试** | 30% | 关键用户流程 |
| **性能测试** | 100% | 关键性能指标 |

### 测试工具

| 类型 | 工具 | 用途 |
|------|------|------|
| **单元测试** | Jest (前端) | 组件测试 |
| | cargo test (后端) | Rust 代码测试 |
| **集成测试** | Testing Library | 组件集成测试 |
| | Tauri Test | Tauri 命令测试 |
| **E2E 测试** | Playwright | 端到端测试 |
| **性能测试** | Lighthouse | 前端性能 |
| | criterion (Rust) | 后端性能 |

### 关键测试场景

1. **创建项目流程**
   - 输入创意 → 创建项目 → 查看会话 → 验证工件生成

2. **恢复会话流程**
   - 选择已完成会话 → 恢复 → 新会话创建 → 验证状态继承

3. **修改项目流程**
   - 选择基础会话 → 输入修改需求 → 执行修改 → 验证代码变更

4. **实时日志流**
   - 启动项目 → 查看实时日志 → 验证日志完整性

5. **文件搜索**
   - 打开搜索框 → 输入关键词 → 验证搜索结果 → 打开文件

6. **代码格式化**
   - 编辑代码 → 触发格式化 → 验证格式化结果

7. **Git 操作**
   - 修改文件 → 查看 Diff → 提交代码 → 验证提交成功

---

## 📊 性能指标

### 性能目标

| 指标 | 目标值 | 当前值 | 说明 |
|------|--------|--------|------|
| **首屏加载时间** | < 2s | 1.5s | 应用启动时间 |
| **文件树渲染** | < 100ms (1000 files) | 200ms | 大项目性能 |
| **代码编辑器加载** | < 500ms | 400ms | Monaco Editor |
| **预览启动时间** | < 3s | 2.5s | 预览服务器 |
| **日志推送延迟** | < 100ms | N/A | 实时日志流 |
| **内存占用** | < 500MB | 450MB | 运行时内存 |
| **包体积** | < 100MB | 80MB | 安装包大小 |

### 性能优化策略

1. **前端优化**
   - 代码分割和懒加载
   - 虚拟滚动
   - 图片和资源压缩
   - 缓存策略

2. **后端优化**
   - 异步处理
   - 连接池
   - 批量操作
   - 内存优化

3. **网络优化**
   - WebSocket 长连接
   - 事件压缩
   - 增量更新

---

## 🔐 安全考虑

### 安全策略

1. **文件系统安全**
   - 路径遍历防护
   - 访问权限检查
   - 文件类型验证

2. **命令执行安全**
   - 命令白名单
   - 参数验证
   - 超时控制

3. **数据安全**
   - 敏感数据加密
   - 本地存储保护
   - 日志脱敏

4. **网络安全**
   - 预览服务器隔离
   - 端口管理
   - 访问控制

### 安全测试

- ✅ 路径遍历测试
- ✅ 命令注入测试
- ✅ XSS 防护测试
- ✅ CSRF 防护测试
- ✅ 文件上传测试

---

## 📚 文档计划

### 文档类型

| 文档类型 | 目标 | 内容 |
|---------|------|------|
| **用户文档** | 帮助用户快速上手 | 快速开始、功能介绍、常见问题 |
| **开发文档** | 指导开发者贡献代码 | 架构说明、API 文档、开发指南 |
| **插件文档** | 指导插件开发 | 插件 API、插件示例、最佳实践 |
| **API 文档** | 描述所有 API 接口 | 命令列表、事件列表、数据类型 |

### 文档工具

- **用户文档**: Docusaurus
- **开发文档**: rustdoc (Rust) + JSDoc (JavaScript)
- **API 文档**: OpenAPI Specification

---

## 🎯 成功指标

### 用户体验指标

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| **任务完成率** | > 90% | 用户调研 |
| **用户满意度** | > 4.0/5.0 | 用户问卷 |
| **学习曲线** | < 30min | 新用户测试 |
| **错误率** | < 5% | 错误日志分析 |

### 技术指标

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| **测试覆盖率** | > 70% | 测试工具 |
| **性能达标率** | 100% | 性能测试 |
| **代码质量** | A 级 | SonarQube |
| **安全漏洞** | 0 | 安全扫描 |

### 业务指标

| 指标 | 目标值 | 测量方法 |
|------|--------|---------|
| **日活跃用户** | > 1000 | 使用统计 |
| **项目创建数** | > 5000/月 | 使用统计 |
| **用户留存率** | > 60% (30天) | 使用统计 |
| **用户推荐率** | > 70% | 用户调研 |

---

## 🚦 风险与应对

### 技术风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|---------|
| **性能问题** | 高 | 中 | 提前性能测试，优化关键路径 |
| **兼容性问题** | 中 | 中 | 多平台测试，提供兼容性方案 |
| **第三方依赖变更** | 中 | 低 | 版本锁定，定期更新测试 |
| **安全漏洞** | 高 | 低 | 安全审计，及时修复 |

### 项目风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|---------|
| **需求变更** | 中 | 高 | 灵活的迭代计划，优先级管理 |
| **资源不足** | 高 | 中 | 合理排期，及时调整 |
| **延期交付** | 中 | 中 | 里程碑管理，风险预警 |

### 用户风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|---------|
| **用户不接受** | 高 | 中 | 用户调研，快速反馈 |
| **学习成本高** | 中 | 中 | 完善文档，提供教程 |
| **功能不满足需求** | 高 | 低 | 需求分析，灵活架构 |

---

## 📞 后续支持与维护

### 维护计划

1. **Bug 修复**
   - 24小时内响应严重 Bug
   - 7天内修复一般 Bug
   - 定期发布补丁版本

2. **功能迭代**
   - 每月发布小版本更新
   - 每季度发布大版本更新
   - 持续收集用户反馈

3. **性能优化**
   - 定期性能评估
   - 持续优化关键路径
   - 监控性能指标

4. **安全更新**
   - 及时修复安全漏洞
   - 定期安全审计
   - 安全公告发布

### 社区建设

1. **用户社区**
   - Discord 服务器
   - GitHub Discussions
   - 用户反馈渠道

2. **开发者社区**
   - 贡献指南
   - 代码规范
   - Pull Request 流程

3. **文档维护**
   - 定期更新文档
   - 用户案例分享
   - 最佳实践分享

---

## 📝 附录

### A. 术语表

| 术语 | 说明 |
|------|------|
| **Agent** | AI 智能体，负责特定阶段的任务 |
| **Session** | 会话，一个独立的开发周期 |
| **Artifact** | 工件，AI 生成的文档和代码 |
| **HITL** | Human-in-the-Loop，人机协作 |
| **Pipeline** | 管道，完整的开发流程 |
| **EventBus** | 事件总线，用于事件通信 |

### B. 参考资料

- [GUI_ARCHITECTURE_PLAN.md](./GUI_ARCHITECTURE_PLAN.md) - GUI 架构规划
- [GUI_ENHANCEMENT_PLAN.md](./GUI_ENHANCEMENT_PLAN.md) - GUI 增强方案
- [README_zh.md](./README_zh.md) - 项目 README
- [litho.docs/](./litho.docs/) - 完整项目文档

### C. 联系方式

- **项目主页**: https://github.com/sopaco/cowork-forge
- **Issues**: https://github.com/sopaco/cowork-forge/issues
- **Discussions**: https://github.com/sopaco/cowork-forge/discussions

---

**文档版本**: 1.0  
**最后更新**: 2026-02-02  
**维护者**: Cowork Forge Team