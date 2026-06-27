/// Compare versions, return true if latest > current
pub fn compare_versions(current: &str, latest: &str) -> bool {
    if current == "0" {
        return true;
    }

    fn parse_version(v: &str) -> (u32, u32, u32) {
        let main = v.split('-').next().unwrap_or(v);
        let parts: Vec<&str> = main.split('.').collect();
        let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    }

    let (c_major, c_minor, c_patch) = parse_version(current);
    let (l_major, l_minor, l_patch) = parse_version(latest);

    (l_major, l_minor, l_patch) > (c_major, c_minor, c_patch)
}

pub fn get_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else {
        "linux".to_string()
    }
}

pub fn get_arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "amd64".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "arm64".to_string()
    } else if cfg!(target_arch = "x86") {
        "386".to_string()
    } else if cfg!(target_arch = "arm") {
        "arm".to_string()
    } else {
        "amd64".to_string()
    }
}

#[allow(dead_code)]
pub fn parse_version_from_location(location: &str) -> Option<String> {
    location
        .rsplit('/')
        .next()
        .and_then(|tag| tag.strip_prefix('v'))
        .map(|v| v.to_string())
}
