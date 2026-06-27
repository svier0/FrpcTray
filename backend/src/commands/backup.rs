use std::fs;
use crate::config::*;

#[tauri::command]
pub async fn export_backup(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let timestamp = chrono::Local::now().format("%y%m%d%H%M%S").to_string();
    let file_name = format!("frpc-{}.zip", timestamp);

    let path = app
        .dialog()
        .file()
        .add_filter("Zip", &["zip"])
        .set_file_name(&file_name)
        .blocking_save_file();

    let path = match path {
        Some(p) => p.as_path().ok_or_else(|| "路径无效".to_string())?.to_path_buf(),
        None => return Err("用户取消了保存".to_string()),
    };

    let dir = get_config_dir();
    let zip_file = fs::File::create(&path)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("读取配置目录失败: {}", e))?
        .flatten()
        .filter(|e| e.path().is_file())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let file_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        zip.start_file(&name, options.clone())
            .map_err(|e| format!("添加文件到压缩包失败: {}", e))?;
        let mut file = fs::File::open(&file_path)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        std::io::copy(&mut file, &mut zip)
            .map_err(|e| format!("写入压缩包失败: {}", e))?;
    }

    let _ = zip.finish().map_err(|e| format!("完成压缩失败: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn restore_backup(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;

    let path = app
        .dialog()
        .file()
        .add_filter("Zip", &["zip"])
        .blocking_pick_file();

    let path = match path {
        Some(p) => p.as_path().ok_or_else(|| "路径无效".to_string())?.to_path_buf(),
        None => return Err("用户取消了选择".to_string()),
    };

    let zip_file = fs::File::open(&path)
        .map_err(|e| format!("打开文件失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("读取压缩包失败: {}", e))?;

    let dir = get_config_dir();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取压缩包条目失败: {}", e))?;
        let name = entry.name().to_string();
        if !name.ends_with(".toml") {
            continue;
        }
        let dest = dir.join(&name);
        let mut out = fs::File::create(&dest)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        std::io::copy(&mut entry, &mut out)
            .map_err(|e| format!("解压文件失败: {}", e))?;
    }

    Ok(())
}
