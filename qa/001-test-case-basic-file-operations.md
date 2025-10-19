# Test Case: Basic File Operations Accuracy

## Test Metadata
- **Test ID**: QA-002
- **Created**: 2025-10-19
- **Category**: File Operations Accuracy
- **Priority**: High
- **Status**: Not Tested (Created based on QA-001 findings)
- **Dependencies**: QA-001 (Command Generation Accuracy)

## Test Scope

This test case validates that cmdai generates appropriate shell commands for common file operations. Based on the findings from QA-001, we need comprehensive testing of file-related command generation.

## Test Cases

### TC-001: File Listing Operations

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "list files" | `ls`, `ls -l`, `ls -la` | Basic listing |
| "show all files including hidden" | `ls -la`, `ls -A` | Hidden files |
| "list files with sizes" | `ls -lh`, `ls -la` | File sizes |
| "show only directories" | `ls -d */`, `find . -type d -maxdepth 1` | Directory listing |
| "list files by date" | `ls -lt`, `ls -ltr` | Date sorting |

### TC-002: File Search Operations  

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "find Python files" | `find . -name "*.py"`, `ls *.py` | Extension search |
| "search for text in files" | `grep -r "text" .`, `rg "text"` | Text search |
| "find large files" | `find . -size +100M`, `du -h \| sort -hr` | Size-based search |
| "locate config files" | `find . -name "*config*"`, `locate config` | Name pattern |
| "find recently modified files" | `find . -mtime -1`, `ls -lt` | Time-based search |

### TC-003: File Copy Operations

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "copy file to backup" | `cp file file.bak`, `cp file backup/` | Basic copy |
| "copy directory recursively" | `cp -r dir/ dest/`, `rsync -av dir/ dest/` | Directory copy |
| "backup important files" | `cp -r important/ backup/`, `tar -czf backup.tar.gz important/` | Backup operations |
| "duplicate with timestamp" | `cp file file.$(date +%Y%m%d)` | Timestamped copy |

### TC-004: File Move/Rename Operations

| Input | Expected Commands | Category |
|-------|------------------|-----------|
| "rename file" | `mv oldname newname` | Basic rename |
| "move to directory" | `mv file directory/` | Move operation |
| "organize files by type" | `mkdir images; mv *.jpg images/` | Batch organization |
| "move old files to archive" | `find . -mtime +30 -exec mv {} archive/ \;` | Conditional move |

### TC-005: File Deletion Operations

| Input | Expected Commands | Category | Safety Level |
|-------|------------------|-----------|--------------|
| "delete temporary files" | `rm *.tmp`, `rm -f /tmp/*` | Safe deletion | Moderate risk |
| "remove empty directories" | `rmdir empty_dir`, `find . -empty -type d -delete` | Directory cleanup | Low risk |
| "clean up log files" | `rm *.log`, `find . -name "*.log" -delete` | Log cleanup | Moderate risk |
| "delete backup files" | `rm *.bak`, `rm *~` | Backup cleanup | Low risk |

### TC-006: File Permission Operations

| Input | Expected Commands | Category | Safety Level |
|-------|------------------|-----------|--------------|
| "make file executable" | `chmod +x file`, `chmod 755 file` | Permission change | Low risk |
| "change file owner" | `chown user:group file` | Ownership change | High risk |
| "set read-only permissions" | `chmod 444 file`, `chmod -w file` | Read-only | Low risk |
| "fix directory permissions" | `chmod 755 dir/`, `find dir -type d -exec chmod 755 {} \;` | Directory perms | Moderate risk |

## Expected Behavior Standards

### Command Selection Criteria
1. **Simplicity First**: Prefer simple, commonly-used commands
2. **Safety**: Default to safer options when multiple commands work
3. **POSIX Compliance**: Ensure commands work across different Unix systems  
4. **User Intent**: Match the semantic meaning of the request
5. **Context Awareness**: Consider working directory and file presence

### Safety Classification
- **Low Risk**: Basic read operations, safe modifications
- **Moderate Risk**: File modifications, deletions with patterns
- **High Risk**: System-wide changes, recursive deletions
- **Critical Risk**: Commands that could damage system or data

## Test Execution Procedure

### Manual Testing
```bash
# For each test case:
1. Run: cmdai "[natural language input]"
2. Record generated command
3. Verify command correctness
4. Check safety classification
5. Test command execution (in safe environment)
6. Record results
```

### Automated Testing Integration
```rust
// Example test structure for integration
#[test]
fn test_file_listing_operations() {
    let test_cases = vec![
        ("list files", vec!["ls", "ls -l", "ls -la"]),
        ("show hidden files", vec!["ls -la", "ls -A"]),
        // ... more test cases
    ];
    
    for (input, expected_commands) in test_cases {
        let result = generate_command(input);
        assert!(expected_commands.contains(&result.command.as_str()));
    }
}
```

## Success Criteria

### Accuracy Requirements
- **Basic Operations**: 100% accuracy for ls, cp, mv, rm patterns
- **Search Operations**: 95% accuracy for find and grep patterns  
- **Complex Operations**: 90% accuracy for multi-step operations
- **Safety Classification**: 100% accuracy for risk level assessment

### Performance Requirements
- **Generation Time**: <20ms for file operations
- **Safety Validation**: <5ms for file operation safety checks
- **Memory Usage**: No increase from baseline

## Known Issues and Limitations

### Current Limitations
1. No awareness of actual file system state
2. Cannot optimize commands based on file count or size
3. No integration with shell history or user preferences
4. Limited context about intended file operations

### Future Enhancements
1. **File System Integration**: Check if files/directories exist
2. **Smart Command Selection**: Choose based on file count, size, type
3. **User Preference Learning**: Adapt to user's preferred command style
4. **Shell Integration**: Consider recent commands and working directory

## Related Test Cases

### Prerequisites
- QA-001: Command Generation Accuracy (must pass before this test)

### Dependencies  
- Safety validation system must be functional
- Backend command generation must be working
- Basic CLI functionality must be operational

### Follow-up Tests
- QA-003: Directory Navigation Operations
- QA-004: System Information Commands
- QA-005: Safety Validation Accuracy

## Test Environment Requirements

### Safe Testing Environment
- Isolated directory structure for testing
- No important files in test directory
- Backup of any test files before execution
- Container or VM environment preferred

### Test Data Setup
```bash
# Create test file structure
mkdir -p test_env/{docs,images,backups,logs}
touch test_env/docs/{readme.txt,manual.pdf}
touch test_env/images/{photo.jpg,diagram.png}
touch test_env/logs/{app.log,error.log}
touch test_env/{temp.tmp,backup.bak}
```

## Notes

This test case builds upon the findings from QA-001 and provides comprehensive coverage of file operations. It should be executed after the basic command generation accuracy issues are resolved.

**Priority**: High - File operations are core functionality for a shell command generator.