# ReadTime - Implementation Plan

## Overview
A command-line tool for estimating reading time of text documents. Can operate on single files or recursively on directories, with hierarchical tree-based output similar to `eza --tree`.

## Research: Reading Speed Default
Based on research, the average adult reading speed is approximately **200-250 words per minute**. Using **200 wpm** as the default is appropriate as it's:
- Conservative (better to over-estimate than under-estimate)
- Well-documented standard
- Accounts for comprehension, not just skimming
- Good for technical documentation which tends to be read slower

**Recommendation**: Use 200 wpm as default, but allow override via command-line argument.

## Core Requirements

### 1. Word Counting
- Count words in text files
- Handle various text formats (plain text, markdown, etc.)
- Strip markdown formatting to get accurate word counts (headings, links, code blocks, etc.)

### 2. File Discovery
- Single file mode: Process one file
- Directory mode: Recursively walk directory tree
- Filter by file extensions
- Default extensions: `.md`, `.txt`, `.rst`, `.adoc`, `.org`
- Exclude programming files by default (`.py`, `.ts`, `.js`, `.rs`, etc.)
- Allow user to specify custom extensions

### 3. Reading Time Calculation
- Formula: `ceiling(word_count / words_per_minute)`
- Always round up to nearest minute
- Default wpm: 200
- Allow override via command-line option

### 4. Output Format
- Tree-based hierarchical view (like `eza --tree`)
- Directories show rolled-up total of all files within
- Files show individual reading time
- Format: `<tree symbol> <name> <reading_time>`
- Reading time format: `X min` (rounded up)
- Example:
  ```
  docs/
  ├── api/                     15 min
  │   ├── authentication.md     5 min
  │   └── endpoints.md         10 min
  ├── guides/                  25 min
  │   ├── quickstart.md        10 min
  │   └── advanced.md          15 min
  └── readme.md                 3 min
  Total: 43 min
  ```

## Architecture

### Module Structure
```
src/
├── main.rs           # Entry point, error handling
├── cli.rs            # Command-line argument parsing
├── config.rs         # Configuration (if needed)
├── counter.rs        # Word counting logic
├── walker.rs         # File system traversal
├── tree.rs           # Tree rendering
└── estimator.rs      # Reading time calculation
```

### Data Structures

#### FileInfo
```rust
struct FileInfo {
    path: PathBuf,
    word_count: usize,
    read_time_minutes: usize,
}
```

#### DirectoryNode
```rust
enum Node {
    File {
        name: String,
        word_count: usize,
        read_time_minutes: usize,
    },
    Directory {
        name: String,
        children: Vec<Node>,
        total_read_time_minutes: usize,
    }
}
```

## Command-Line Interface

### Command Structure
```bash
readtime [OPTIONS] <PATH>
```

### Arguments
- `<PATH>` - File or directory path (required)

### Options
- `-w, --wpm <wpm>` - Words per minute (default: 200)
- `-e, --extensions <ext>...` - File extensions to include (default: md,txt,rst,adoc,org)
- `--add-extensions <ext>...` - Add extensions to default list
- `-c, --config <path>` - Path to config file
- `-v, --verbose` - Enable verbose output

### Examples
```bash
# Single file
readtime readme.md

# Directory (default extensions)
readtime docs/

# Directory with custom wpm
readtime --wpm 250 docs/

# Custom extensions
readtime --extensions md,txt,rst documentation/

# Add extensions to default
readtime --add-extensions tex,latex academic/

# Verbose mode
readtime -v docs/
```

## Implementation Steps

### Phase 1: Core Functionality
1. **Update cli module** (`cli.rs`)
   - Add path argument
   - Add --wpm option
   - Add --extensions option
   - Add --add-extensions option

2. **Create word counter module** (`counter.rs`)
   - Implement basic word counting
   - Handle utf-8 text
   - Strip markdown formatting for accurate counts
   - Use `wc` command as baseline for testing

3. **Create file walker module** (`walker.rs`)
   - Implement single file processing
   - Implement recursive directory traversal
   - Filter by extension
   - Handle symlinks appropriately
   - Skip hidden files/directories
   - Error handling with eyre

4. **Create estimator module** (`estimator.rs`)
   - Calculate reading time from word count
   - Round up to nearest minute
   - Handle configurable wpm

### Phase 2: Tree Rendering
5. **Create tree renderer module** (`tree.rs`)
   - Build tree data structure from file list
   - Calculate rolled-up totals for directories
   - Render with box-drawing characters
   - Handle proper indentation
   - Display reading times aligned to the right

6. **Integration in main.rs**
   - Wire up all modules
   - Handle single file vs directory mode
   - Output to stdout
   - Comprehensive error handling with eyre

### Phase 3: Polish
7. **Testing & Validation**
   - Test with various file structures
   - Test with different extensions
   - Validate word counts against `wc`
   - Test error cases

8. **Documentation**
   - Update readme with usage examples
   - Add comments to code
   - Document any edge cases

## Technical Considerations

### Word Counting Strategy
- Use simple whitespace-based splitting for plain text
- For markdown: strip formatting but keep content
  - Remove code blocks (``` and ~~~)
  - Remove inline code (`code`)
  - Convert links `[text](url)` to just `text`
  - Remove heading markers (#)
  - Keep list content
- Could use `pulldown-cmark` for markdown parsing
- Alternative: pipe through `mdcat` or `pandoc` if available

### File System Traversal
- Use `walkdir` crate for directory traversal
- Respect `.gitignore` patterns? (optional, discuss with user)
- Handle permission errors gracefully
- Skip binary files

### Performance
- For large directories, consider parallel processing
- Cache word counts if processing same files multiple times
- Stream output for large trees

### Error Handling
- All errors use `eyre::Result`
- No `.unwrap()` calls
- Provide context with `.context()` or `.wrap_err()`
- Graceful degradation where possible

## Dependencies to Add
```toml
walkdir = "2"           # Directory traversal
unicode-width = "0.1"   # Proper text width calculation
```

Optional (for markdown parsing):
```toml
pulldown-cmark = "0.9"  # Markdown parsing
```

## Configuration File Schema
The existing `readtime.yml` can be extended:
```yaml
# Default words per minute
wpm: 200

# Default file extensions
extensions:
  - md
  - txt
  - rst
  - adoc
  - org

# Whether to include hidden files
include_hidden: false
```

## Edge Cases to Handle
1. Empty files (0 minutes)
2. Very large files (don't load entire file into memory)
3. Binary files mixed with text files
4. Permission denied errors
5. Symlink loops
6. Files with no extension
7. Non-utf-8 files
8. Very deep directory structures

## Success Criteria
- ✅ Accurate word counting
- ✅ Proper tree visualization
- ✅ Configurable wpm
- ✅ Extension filtering
- ✅ No panics/unwraps
- ✅ Clear error messages
- ✅ Works on single files and directories
- ✅ Rolled-up directory totals
- ✅ Clean, maintainable code

## Future Enhancements (Out of Scope)
- html file support
- pdf file support
- Different reading speed presets (skim, normal, careful)
- json/csv output format
- Language-specific reading speeds
- Exclude patterns (like .gitignore)
- Progress bar for large directories

