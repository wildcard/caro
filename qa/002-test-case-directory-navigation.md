# Test Case: Directory Navigation Operations

## Test Metadata
- **Test ID**: QA-003
- **Created**: 2025-10-19
- **Category**: Directory Navigation Accuracy
- **Priority**: Medium
- **Status**: Not Tested (Created based on QA-001 findings)
- **Dependencies**: QA-001, QA-002

## Test Scope

This test case validates that cmdai generates appropriate shell commands for directory navigation, creation, and management operations. Directory operations are fundamental to shell usage and require high accuracy.

## Test Cases

### TC-001: Basic Navigation

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "go to home directory" | `cd ~`, `cd $HOME`, `cd` | Home navigation |
| "change to parent directory" | `cd ..` | Parent navigation |
| "go back to previous directory" | `cd -` | Previous directory |
| "navigate to root" | `cd /` | Root navigation |
| "show current directory" | `pwd` | Current location |

### TC-002: Path Navigation

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "go to documents folder" | `cd ~/Documents`, `cd Documents` | Named directory |
| "navigate to /usr/local/bin" | `cd /usr/local/bin` | Absolute path |
| "change to relative path src/main" | `cd src/main` | Relative path |
| "go to directory with spaces" | `cd "My Documents"`, `cd 'My Documents'` | Quoted paths |

### TC-003: Directory Creation

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "create new directory" | `mkdir newdir` | Basic creation |
| "make directory with parents" | `mkdir -p path/to/dir` | Recursive creation |
| "create multiple directories" | `mkdir dir1 dir2 dir3` | Multiple creation |
| "make backup directory" | `mkdir backup` | Named creation |
| "create project structure" | `mkdir -p project/{src,tests,docs}` | Structure creation |

### TC-004: Directory Listing and Information

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "show directory contents" | `ls`, `ls -la` | Basic listing |
| "list directories only" | `ls -d */`, `find . -maxdepth 1 -type d` | Directory filter |
| "show directory tree" | `tree`, `find . -type d` | Tree structure |
| "display directory sizes" | `du -h`, `du -sh *` | Size information |
| "count files in directory" | `ls -1 \| wc -l`, `find . -maxdepth 1 -type f \| wc -l` | File counting |

### TC-005: Directory Search and Find

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "find directory by name" | `find . -name "dirname" -type d` | Name search |
| "locate empty directories" | `find . -empty -type d` | Empty search |
| "search for hidden directories" | `find . -name ".*" -type d` | Hidden search |
| "find large directories" | `du -h \| sort -hr \| head` | Size-based search |
| "find recently created directories" | `find . -type d -newerct "1 day ago"` | Time-based search |

### TC-006: Directory Operations

| Input | Expected Commands | Category | Safety Level |
|-------|------------------|-----------|--------------|
| "copy directory" | `cp -r sourcedir destdir` | Directory copy | Low risk |
| "move directory" | `mv olddir newdir` | Directory move | Moderate risk |
| "remove empty directory" | `rmdir emptydir` | Safe removal | Low risk |
| "delete directory and contents" | `rm -rf dirname` | Dangerous removal | High risk |
| "rename directory" | `mv oldname newname` | Directory rename | Low risk |

### TC-007: Working Directory Context

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "go up two levels" | `cd ../..` | Relative navigation |
| "navigate to sibling directory" | `cd ../sibling` | Sibling navigation |
| "return to project root" | `cd $(git rev-parse --show-toplevel)` | Context-aware |
| "find project directory" | `find . -name ".git" -type d \| head -1` | Project detection |

## Expected Behavior Standards

### Path Handling
1. **Absolute vs Relative**: Choose appropriate path type based on context
2. **Quote Handling**: Properly quote paths with spaces or special characters
3. **Tilde Expansion**: Use `~` for home directory references when appropriate
4. **Variable Usage**: Use `$HOME`, `$PWD` when semantically clear

### Safety Considerations
1. **Destructive Operations**: Require confirmation for `rm -rf` operations
2. **Path Validation**: Avoid dangerous paths like `/`, `/usr`, `/bin`
3. **Recursive Operations**: Clear indication when operations are recursive
4. **Permission Awareness**: Consider permission requirements

## Test Execution Procedure

### Manual Testing
```bash
# Setup test environment
mkdir -p test_dirs/{level1/{level2a,level2b},empty,large}
touch test_dirs/level1/level2a/file.txt
touch test_dirs/large/{file1,file2,file3}

# For each test case:
1. cd test_dirs  # Start in known location
2. Run: cmdai "[natural language input]"
3. Record generated command
4. Verify command correctness
5. Test command execution
6. Record results and any issues
```

### Context-Aware Testing
```bash
# Test context awareness
cd ~/Documents
cmdai "go to parent directory"  # Should generate: cd ..

cd /usr/local
cmdai "navigate to bin"  # Should generate: cd bin

cd ~/Projects/myproject
cmdai "go to project root"  # Should detect git root if available
```

## Success Criteria

### Accuracy Requirements
- **Basic Navigation**: 100% accuracy for cd, pwd operations
- **Path Resolution**: 95% accuracy for relative/absolute path selection
- **Directory Creation**: 100% accuracy for mkdir operations
- **Directory Listing**: 95% accuracy for ls and find operations
- **Safety Classification**: 100% accuracy for dangerous operations

### Contextual Awareness
- **Relative Paths**: Correctly use relative paths when appropriate
- **Home Directory**: Properly expand `~` in relevant contexts
- **Current Directory**: Account for current working directory in suggestions

## Test Scenarios

### Scenario 1: Project Navigation
```bash
# User is in a software project
cd ~/Projects/myapp
cmdai "go to source directory"     # Should suggest: cd src
cmdai "navigate to tests"          # Should suggest: cd tests  
cmdai "return to project root"     # Should use git/project detection
```

### Scenario 2: System Administration
```bash
# System admin tasks
cd /etc
cmdai "check configuration directory"  # Should suggest: ls -la
cmdai "go to log directory"            # Should suggest: cd /var/log
cmdai "navigate to system binaries"    # Should suggest: cd /usr/bin
```

### Scenario 3: File Organization
```bash
# File organization workflow
cd ~/Downloads
cmdai "create organization structure"   # Should create logical directories
cmdai "move to documents folder"       # Should navigate appropriately
cmdai "make backup directory"          # Should create backup location
```

## Known Issues and Limitations

### Current Limitations
1. No awareness of actual directory structure
2. Cannot verify directory existence before navigation
3. No integration with shell history or bookmarks
4. Limited context about user's typical directory patterns

### Context Dependencies
1. **Working Directory**: Commands depend on current location
2. **File System State**: Accuracy depends on actual directory existence
3. **User Preferences**: No learning of user's navigation patterns
4. **Project Context**: No detection of project types (git, npm, cargo, etc.)

## Future Enhancements

### File System Integration
- **Existence Checking**: Verify directories exist before suggesting navigation
- **Completion Suggestions**: Provide directory completions based on actual filesystem
- **Smart Path Resolution**: Resolve relative paths based on current context

### Context Awareness
- **Project Detection**: Identify project roots and suggest appropriate navigation
- **Bookmark Integration**: Learn from user's frequently accessed directories
- **Shell History**: Consider recently visited directories
- **Directory Patterns**: Learn user's organizational patterns

## Related Test Cases

### Prerequisites
- QA-001: Command Generation Accuracy
- QA-002: Basic File Operations

### Follow-up Tests
- QA-004: System Information Commands
- QA-005: Safety Validation Accuracy
- QA-006: Development Workflow Operations

## Error Conditions

### Invalid Paths
| Input | Expected Behavior | Safety |
|-------|------------------|--------|
| "go to non-existent directory" | Generate valid command but warn if possible | Low risk |
| "navigate to /dev/null" | Block or warn about special paths | High risk |
| "cd to binary file" | Generate command but indicate potential error | Medium risk |

### Permission Issues
| Input | Expected Behavior | Safety |
|-------|------------------|--------|
| "access restricted directory" | Generate command with permission awareness | Medium risk |
| "create directory in system path" | Warn about permission requirements | High risk |

## Notes

Directory navigation is fundamental to shell usage and directly impacts user productivity. This test case ensures that cmdai can handle the full spectrum of directory operations accurately and safely.

**Special Attention**: Context awareness will be crucial for this category, as the same natural language input may require different commands based on current working directory and file system state.