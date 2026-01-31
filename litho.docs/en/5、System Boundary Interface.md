# System Boundary Interface Documentation

This document describes the system's external invocation interfaces, including CLI commands, API endpoints, configuration parameters, and other boundary mechanisms.

## Command Line Interface (CLI)

### cowork new

**Description**: Create a new project. Initialize new project directory structure based on user-provided idea description, create session record, and execute complete AI-driven development process (requirements analysis → system design → task planning → code implementation → quality check → project delivery)

**Source File**: `crates/cowork-cli/src/main.rs`

**Arguments**:

- `idea` (String): required - Project idea/description (wrap descriptions with spaces in quotes)

**Options**:

- `verbose, v`(boolean): optional - Output detailed logs, including ADK internal debug information (default: `false`)
- `config, c`(filepath): optional - Configuration file path, default is config.toml in current directory
- `stream, s`(boolean): optional - Enable LLM streaming output, real-time display of AI thinking process (default: `false`)

**Usage Examples**:

```bash
cowork new "Create an elementary school math intelligent test paper generation system"
```

```bash
cowork new "Develop a personal blog website supporting Markdown"
```

### cowork resume

**Description**: Resume existing project from checkpoint. Continue the last successful or in-progress session, supporting specifying base session ID or automatically selecting the latest session. Used to handle interrupted workflows or continue development after completed stages

**Source File**: `crates/cowork-cli/src/main.rs`

**Options**:

- `verbose, v`(boolean): optional - Whether to output detailed logs (default: `false`)
- `config, c`(filepath): optional - Configuration file path
- `stream, s`(boolean): optional - Whether to enable streaming output (default: `false`)
- `base, b`(string): optional - Specified base session ID (optional). If omitted, defaults to latest successful session; if no successful session, tries latest in-progress session

**Usage Examples**:

```bash
cowork resume
```

```bash
cowork resume --base session-1703123456
```

```bash
cowork resume -b session-1703123456
```

### cowork revert

**Description**: Restart project from specified stage. Allows users to rollback to requirements (PRD), design (Design), planning (Plan), coding (Coding), check (Check), or delivery (Delivery) stage for re-execution. Supports auto mode to automatically identify problem stage

**Source File**: `crates/cowork-cli/src/main.rs`

**Options**:

- `verbose, v`(boolean): optional - Whether to output detailed logs (default: `false`)
- `config, c`(filepath): optional - Configuration file path
- `stream, s`(boolean): optional - Whether to enable streaming output (default: `false`)
- `from, f`(string): required - Restart start stage, optional values: prd, design, plan, coding, check, delivery, auto

**Usage Examples**:

```bash
cowork revert --from design
```

```bash
cowork revert -f prd
```

```bash
cowork revert --from auto
```

### cowork modify

**Description**: Perform incremental modification on existing project. Create change request based on previous successful session, execute modification workflow, automatically detect file changes (add/delete/modify) and generate patch metadata. Used for feature iteration, bug fixes, or requirement adjustments

**Source File**: `crates/cowork-cli/src/main.rs`

**Arguments**:

- `idea` (string): required - Change requirement description

**Options**:

- `verbose, v`(boolean): optional - Whether to output detailed logs (default: `false`)
- `config, c`(filepath): optional - Configuration file path
- `stream, s`(boolean): optional - Whether to enable streaming output (default: `false`)
- `base, b`(string): optional - Base session ID (default is latest successful session)

**Usage Examples**:

```bash
cowork modify "Add user login functionality"
```

```bash
cowork modify "Fix database connection pool issue" --base session-1703123456
```

### cowork status

**Description**: Display current project status. Including project metadata, session history, and latest session artifact generation status (requirements document, feature list, design document, task progress, etc.). Optionally display all session details

**Source File**: `crates/cowork-cli/src/main.rs`

**Options**:

- `verbose, v`(boolean): optional - Whether to output detailed logs (default: `false`)
- `config, c`(filepath): optional - Configuration file path
- `stream, s`(boolean): optional - Whether to enable streaming output (default: `false`)
- `sessions, s`(boolean): optional - Display detailed list of all sessions (default: `false`)

**Usage Examples**:

```bash
cowork status
```

```bash
cowork status --sessions
```

### cowork init

**Description**: Initialize configuration file in current directory. Create default config.toml template file, including LLM API connection parameter configuration (api_base_url, api_key, model_name). Error if file already exists

**Source File**: `crates/cowork-cli/src/main.rs`

**Options**:

- `verbose, v`(boolean): optional - Whether to output detailed logs (default: `false`)
- `config, c`(filepath): optional - Configuration file path
- `stream, s`(boolean): optional - Whether to enable streaming output (default: `false`)

**Usage Examples**:

```bash
cowork init
```

## Integration Suggestions

### Shell/CLI Integration

CLI tool integration guide: As a command line tool, Cowork Forge integrates into development workflows through Shell calls. Supports LLM connection configuration via configuration files or environment variables, can be integrated with CI/CD pipelines, scripts, or IDE task systems.

**Example Code**:

```
#!/bin/bash
# Initialize project configuration
cowork init

# Edit configuration file to fill in API key
sed -i 's/your-api-key-here/actual-api-key/' config.toml

# Create new project (with streaming output and detailed logs)
cowork new "Smart todo application" --verbose --stream

# Check project status
cowork status

# Incremental modification (add functionality)
cowork modify "Add category tag functionality"

# If need to rollback to design stage
cowork revert --from design
```

**Best Practices**:

- Run 'cowork init' before first use to create configuration file, edit to set LLM API endpoint and key
- Use LLM services supporting OpenAI API format (such as vLLM, Ollama, LiteLLM, etc.), configure compatible api_base_url
- Project session data stored in .cowork directory, ensure directory is not deleted or committed to version control (add .gitignore)
- Use --stream flag to get real-time AI output for debugging, can close in production environment to reduce output noise
- For complex projects, execute in stages, use 'revert' to rollback to specified stage when necessary
- Regularly use 'status --sessions' to check session history, clean up failed session records in time

### Configuration Management Integration

LLM service configuration integration: System communicates with LLM service via OpenAI compatible protocol. Need to configure valid API endpoint, key, and model name. Supports local deployment (vLLM/Ollama) and cloud APIs (OpenAI/DeepSeek, etc.).

**Example Code**:

```
# config.toml configuration example
[llm]
api_base_url = "http://localhost:8000/v1"
api_key = "sk-xxx"
model_name = "gpt-4"

# Docker Compose integration example
version: '3.8'
services:
  cowork-forge:
    image: cowork-forge:latest
    environment:
      - LLM_API_BASE_URL=http://vllm:8000/v1
      - LLM_API_KEY=local-key
      - LLM_MODEL_NAME=Qwen-7B
    volumes:
      - ./workspace:/workspace
    working_dir: /workspace
```

**Best Practices**:

- Prioritize config.toml file for production environment configuration, facilitating version control and backup
- Development/testing environment can use environment variables (LLM_API_BASE_URL, LLM_API_KEY, LLM_MODEL_NAME) for quick switching
- Ensure API key security, config.toml should be added to .gitignore, use environment variable injection for sensitive information
- For OpenAI compatible services, api_base_url needs to include /v1 path suffix (e.g., http://localhost:8000/v1)
- Enable rate limiting (built-in 2-second delay) to prevent exceeding LLM service provider quota limits


---

**Analysis Confidence**: 9.5/10