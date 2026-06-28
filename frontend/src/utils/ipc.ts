import { invoke } from "@tauri-apps/api/core";
import i18n from "../i18n";
import type { ServerItem } from "../components/ServerItem.vue";

export interface ProxyItem {
  name: string;
  desc?: string | null;
  enabled: boolean;
  type: string;
  localIP?: string | null;
  localPort: number;
  remotePort?: number | null;
  customDomains?: string[] | null;
  locations?: string[] | null;
}

export async function listServers(): Promise<ServerItem[]> {
  return invoke("list_servers");
}

export async function getServer(id: string): Promise<ServerItem> {
  return invoke("get_server", { id });
}

export async function createServer(server: Omit<ServerItem, "id">): Promise<string> {
  return invoke("create_server", { input: server });
}

export async function updateServer(server: ServerItem): Promise<void> {
  return invoke("update_server", { server });
}

export async function deleteServer(id: string): Promise<void> {
  return invoke("delete_server", { id });
}

export async function reorderServers(ids: string[]): Promise<void> {
  return invoke("reorder_servers", { ids });
}

export async function listProxies(serverId: string): Promise<ProxyItem[]> {
  return invoke("list_proxies", { serverId });
}

export async function createProxy(serverId: string, proxy: ProxyItem): Promise<void> {
  return invoke("create_proxy", { serverId, proxy });
}

export async function updateProxy(serverId: string, oldName: string, proxy: ProxyItem): Promise<void> {
  return invoke("update_proxy", { serverId, oldName, proxy });
}

export async function deleteProxy(serverId: string, name: string): Promise<void> {
  return invoke("delete_proxy", { serverId, name });
}

export async function reorderProxies(serverId: string, names: string[]): Promise<void> {
  return invoke("reorder_proxies", { serverId, names });
}

export interface FrpcVersionInfo {
  current_version: string;
  latest_version: string;
  can_upgrade: boolean;
  platform: string;
  arch: string;
}

export async function getFrpcVersion(): Promise<FrpcVersionInfo> {
  return invoke("get_frpc_version");
}

export async function upgradeFrpc(version: string): Promise<void> {
  return invoke("upgrade_frpc", { version });
}

export async function exportBackup(): Promise<string> {
  return invoke("export_backup");
}

export async function restoreBackup(): Promise<void> {
  return invoke("restore_backup");
}

export interface AppConfig {
  language: string;
  theme: string;
  autostart: boolean;
  silent_launch: boolean;
  auto_run: boolean;
}

export async function getConfig(): Promise<AppConfig> {
  return invoke("get_config");
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}

export interface FrpcRunningStatus {
  server_id: string;
  status: "running" | "stopped" | "error";
  pid: number | null;
  error_message: string | null;
}

export async function startFrpc(serverId: string): Promise<void> {
  return invoke("start_frpc", { serverId });
}

export async function stopFrpc(serverId: string): Promise<void> {
  return invoke("stop_frpc", { serverId });
}

export async function startAllFrpc(): Promise<void> {
  return invoke("start_all_frpc");
}

export async function stopAllFrpc(): Promise<void> {
  return invoke("stop_all_frpc");
}

export async function getAllFrpcStatus(): Promise<FrpcRunningStatus[]> {
  return invoke("get_all_frpc_status");
}

const errorPatterns: Array<{ regex: RegExp; keys: string[]; zhCN: string; zhTW: string; en: string; ja: string }> = [
  { regex: /Unknown config field "([^"]+)"/i, keys: ["name"], zhCN: "未知配置字段 \"{name}\"", zhTW: "未知設定欄位 \"{name}\"", en: "Unknown config field \"{name}\"", ja: "不明な設定フィールド \"{name}\"" },
  { regex: /Proxy name "([^"]+)" already in use/i, keys: ["name"], zhCN: "代理名 \"{name}\" 已被使用", zhTW: "代理名 \"{name}\" 已被使用", en: "Proxy name \"{name}\" already in use", ja: "プロキシ名 \"{name}\" は既に使用されています" },
  { regex: /Port (\d+) already in use/i, keys: ["port"], zhCN: "端口 {port} 已被占用", zhTW: "連接埠 {port} 已被占用", en: "Port {port} already in use", ja: "ポート {port} は既に使用されています" },
  { regex: /Port (\d+) unavailable/i, keys: ["port"], zhCN: "端口 {port} 不可用", zhTW: "連接埠 {port} 不可用", en: "Port {port} unavailable", ja: "ポート {port} が利用できません" },
  { regex: /Config parse error: (.+)/i, keys: ["detail"], zhCN: "配置解析错误：{detail}", zhTW: "設定解析錯誤：{detail}", en: "Config parse error: {detail}", ja: "設定の解析エラー：{detail}" },
  { regex: /TLS certificate verification failed: (.+)/i, keys: ["reason"], zhCN: "TLS 证书验证失败：{reason}", zhTW: "TLS 憑證驗證失敗：{reason}", en: "TLS certificate verification failed: {reason}", ja: "TLS証明書の検証に失敗しました：{reason}" },
  { regex: /TLS error: (.+)/i, keys: ["detail"], zhCN: "TLS 错误：{detail}", zhTW: "TLS 錯誤：{detail}", en: "TLS error: {detail}", ja: "TLSエラー：{detail}" },
  { regex: /Health check failed: (.+)/i, keys: ["detail"], zhCN: "健康检查失败：{detail}", zhTW: "健康檢查失敗：{detail}", en: "Health check failed: {detail}", ja: "ヘルスチェックに失敗しました：{detail}" },
];

const errorExact: Record<string, Record<string, string>> = {
  "Login to server failed": { zhCN: "登录服务器失败", zhTW: "登入伺服器失敗", en: "Login to server failed", ja: "サーバーへのログインに失敗しました" },
  "Token mismatch": { zhCN: "Token 不匹配", zhTW: "Token 不匹配", en: "Token mismatch", ja: "トークンが一致しません" },
  "Connection refused": { zhCN: "连接被拒绝", zhTW: "連線被拒絕", en: "Connection refused", ja: "接続が拒否されました" },
  "Connection reset": { zhCN: "连接被重置", zhTW: "連線被重設", en: "Connection reset", ja: "接続がリセットされました" },
  "Network unreachable": { zhCN: "网络不可达", zhTW: "網路不可達", en: "Network unreachable", ja: "ネットワークに到達できません" },
  "Connection timeout": { zhCN: "连接超时", zhTW: "連線逾時", en: "Connection timeout", ja: "接続タイムアウト" },
  "Reconnect timeout": { zhCN: "重连超时", zhTW: "重連逾時", en: "Reconnect timeout", ja: "再接続タイムアウト" },
  "Control channel closed": { zhCN: "控制通道已关闭", zhTW: "控制通道已關閉", en: "Control channel closed", ja: "制御チャネルが閉じられました" },
  "Permission denied": { zhCN: "权限不足", zhTW: "權限不足", en: "Permission denied", ja: "権限がありません" },
  "Connection closed unexpectedly": { zhCN: "连接意外关闭", zhTW: "連線意外關閉", en: "Connection closed unexpectedly", ja: "接続が予期せず閉じられました" },
};

function getLocale(): string {
  return (i18n.global.locale as any).value || "zh-CN";
}

function getLang(): string {
  const loc = getLocale();
  if (loc.startsWith("zh-TW")) return "zhTW";
  if (loc.startsWith("en")) return "en";
  if (loc.startsWith("ja")) return "ja";
  return "zhCN";
}

export function translateError(error: string | null): string {
  if (!error) return "";
  const lang = getLang();
  for (const { regex, keys, zhCN, zhTW, en, ja } of errorPatterns) {
    const match = error.match(regex);
    if (match) {
      const map = { zhCN, zhTW, en, ja };
      let result = map[lang];
      keys.forEach((key, i) => { result = result.replace(`{${key}}`, match[i + 1]); });
      return result;
    }
  }
  if (errorExact[error]) return errorExact[error][lang] || error;
  return error;
}
