use std::error::Error;
use winreg::RegKey;
use winreg::enums::*;

const REG_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "SSHTunnelManager";

/// 检查是否已设置开机启动
pub fn is_autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = hkcu.open_subkey(REG_KEY_PATH).ok();

    if let Some(key) = path {
        key.get_value::<String, _>(APP_NAME).ok().is_some()
    } else {
        false
    }
}

/// 启用开机启动
pub fn enable_autostart() -> Result<(), Box<dyn Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy().to_string();
    // 路径需要加引号，避免路径中有空格时 Windows 无法正确解析
    let quoted_path = format!("\"{}\"", exe_path_str);

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // 使用 KEY_WRITE 权限创建/打开注册表项
    let (key, _) = hkcu.create_subkey_with_flags(REG_KEY_PATH, KEY_WRITE)?;

    key.set_value(APP_NAME, &quoted_path)?;

    Ok(())
}

/// 禁用开机启动
pub fn disable_autostart() -> Result<(), Box<dyn Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    // 使用 KEY_WRITE 权限打开注册表项
    let path = hkcu.open_subkey_with_flags(REG_KEY_PATH, KEY_WRITE);

    match path {
        Ok(key) => {
            // 删除注册表项
            key.delete_value(APP_NAME)?;
            Ok(())
        }
        Err(e) => {
            // 如果注册表路径不存在，说明没有设置开机启动，直接返回成功
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(Box::new(e))
            }
        }
    }
}