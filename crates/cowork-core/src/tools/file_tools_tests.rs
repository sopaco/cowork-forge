#[cfg(test)]
mod tests {
    use crate::tools::file_tools::{build_gitignore_walker, is_hidden_file};
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_list_directory_non_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // 创建嵌套目录
        fs::create_dir(root.join("dir1")).unwrap();
        fs::write(root.join("dir1/file1.txt"), "file1").unwrap();
        
        fs::create_dir(root.join("dir1/dir2")).unwrap();
        fs::write(root.join("dir1/dir2/file2.txt"), "file2").unwrap();

        // 测试非递归（只列出 dir1，不包含 dir2）
        let walker = build_gitignore_walker(root.join("dir1").to_str().unwrap(), false, false);
        let paths: Vec<String> = walker
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_string_lossy().to_string())
            .collect();

        // 应该包含 file1.txt 和 dir2 目录
        assert!(paths.iter().any(|p: &String| p.contains("file1.txt")));
        assert!(paths.iter().any(|p: &String| p.ends_with("dir2")));
        
        // 不应该包含深层的 file2.txt
        assert!(!paths.iter().any(|p: &String| p.contains("file2.txt")));
    }

    #[test]
    fn test_is_hidden_file() {
        use std::path::PathBuf;
        
        assert!(is_hidden_file(&PathBuf::from(".hidden")));
        assert!(is_hidden_file(&PathBuf::from("/path/to/.hidden")));
        assert!(!is_hidden_file(&PathBuf::from("visible")));
        assert!(!is_hidden_file(&PathBuf::from("/path/to/visible")));
    }

    #[test]
    fn test_walker_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("file.txt"), "content").unwrap();
        fs::create_dir(root.join("dir")).unwrap();
        fs::write(root.join("dir/file.txt"), "content").unwrap();

        // 测试非递归
        let walker = build_gitignore_walker(root.to_str().unwrap(), false, false);
        let count_non_recursive = walker.filter_map(|e| e.ok()).count();
        
        // 测试递归
        let walker = build_gitignore_walker(root.to_str().unwrap(), true, false);
        let count_recursive = walker.filter_map(|e| e.ok()).count();
        
        // 递归应该返回更多文件
        assert!(count_recursive >= count_non_recursive);
    }
}
