# Basic Utility Functions Documentation

## Overview

The Basic Utility Functions module provides lightweight, stateless utility functions that support various core operations throughout the Cowork Forge system. These functions focus on common data extraction and parsing tasks that are used across multiple agents and tools, promoting code reuse and consistency.

## Module Structure

The utility functions are organized into specialized modules:

### 1. PRD Utilities (`src/utils/prd_utils.rs`)

#### `extract_prd_summary()`

**Purpose**: Extracts a human-readable summary from a Product Requirements Document (PRD) artifact for monitoring and logging purposes.

**Function Signature**:
```rust
pub fn extract_prd_summary(prd_artifact: &PRDArtifact) -> String
```

**Parameters**:
- `prd_artifact`: Reference to a PRD artifact containing the document data

**Return Value**: A formatted string containing a Markdown-like summary

**Implementation Details**:

The function processes the PRD data structure and creates a concise summary by:
- Extracting up to 3 goals from the `scope.g` field
- Extracting up to 5 requirements from the `reqs` field
- Formatting the output with appropriate headers and list formatting
- Adding a note when more than 5 requirements exist

**Output Format**:
```
**Goals**:
- Goal 1
- Goal 2
- Goal 3

**Requirements**:
- REQ-001: Description of requirement 1
- REQ-002: Description of requirement 2
- REQ-003: Description of requirement 3
- REQ-004: Description of requirement 4
- REQ-005: Description of requirement 5
... and X more requirements
```

**Usage Context**: Primarily used by the WatchDog agent to provide concise project overviews during long-running workflows without requiring full data inspection.

### 2. Error Extraction Utilities (`src/verification/error_extract.rs`)

#### `extract_paths()`

**Purpose**: Parses stderr/stdout output to extract file paths from error messages across multiple programming languages.

**Function Signature**:
```rust
pub fn extract_paths(text: &str) -> Vec<String>
```

**Parameters**:
- `text`: Raw output text containing error messages

**Return Value**: Vector of unique, normalized file paths found in the error output

**Supported Language Patterns**:

1. **TypeScript/JavaScript & Generic**:
   - Pattern: `path/to/file.ts:line:col`
   - Regex: `[A-Za-z0-9_./\\-]+\.(?:ts|tsx|js|jsx|mjs|cjs|rs|py|go|java|kt|cpp|h|hpp)):(\d+):(\d+)`

2. **Rust**:
   - Pattern: `--> src/main.rs:42:5`
   - Regex: `-->\\s+([A-Za-z0-9_./\\-]+\.rs):\d+:\d+`

3. **Python**:
   - Pattern: `File "main.py"`
   - Regex: `File\\s+"([^"]+\.py)"`

**Implementation Details**:

- **Path Normalization**: All paths are normalized to use forward slashes (`/`) regardless of the original format
- **Duplicate Prevention**: The function ensures only unique file paths are returned
- **Robust Matching**: Uses multiple regex patterns to handle different compiler/linter output formats
- **Fallback Tolerant**: Individual regex failures don't break the entire function

**Helper Functions**:

- `push_unique()`: Adds items to a vector only if they don't already exist
- `normalize_path()`: Converts backslashes to forward slashes for consistency

## Integration Points

### Dependencies
- **regex crate**: Used for pattern matching in error extraction
- **crate::artifacts::***: PRD utilities depend on the artifact data structures

### Consumers
- **WatchDog Agent**: Uses `extract_prd_summary()` for project monitoring
- **Verification Tools**: Uses `extract_paths()` for parsing build/compilation errors
- **Error Analyzer**: Leverages path extraction for intelligent error diagnosis

## Design Principles

### 1. Stateless Operation
All utility functions are pure functions with no internal state, making them thread-safe and predictable.

### 2. Focused Scope
Each utility addresses a single, well-defined problem without attempting to be overly generic.

### 3. Error Resilience
Functions are designed to handle edge cases gracefully, returning empty results rather than failing.

### 4. Performance Considerations
- Regex patterns are compiled once and reused
- String operations are optimized for typical use cases
- Memory allocations are minimized where possible

## Testing Strategy

The module includes comprehensive unit tests:
- **PRD Summary Tests**: Verify correct extraction and formatting of goals and requirements
- **Error Extraction Tests**: Validate path extraction across different language outputs
- **Edge Case Handling**: Tests for empty inputs, malformed data, and boundary conditions

## Usage Examples

### PRD Summary Extraction
```rust
use crate::utils::prd_utils::extract_prd_summary;
use crate::artifacts::*;

let summary = extract_prd_summary(&prd_artifact);
println!("Project Summary: {}", summary);
```

### Error Path Extraction
```rust
use crate::verification::error_extract::extract_paths;

let error_output = "error: expected identifier, found `}`\n  --> src/main.rs:10:5";
let affected_files = extract_paths(error_output);
// Returns: vec!["src/main.rs".to_string()]
```

## Maintenance Considerations

- **Regex Patterns**: May need updates as compiler/linter output formats evolve
- **Language Support**: New programming languages can be added by extending the regex patterns
- **Performance**: Monitor for potential optimizations as usage patterns emerge

The Basic Utility Functions module exemplifies the Cowork Forge philosophy of providing reliable, focused tools that enable higher-level components to operate effectively while maintaining clean separation of concerns.