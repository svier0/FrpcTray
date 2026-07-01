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
  status: "running" | "connecting" | "stopped" | "error";
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

const errorPatterns: Array<{ regex: RegExp; keys: string[]; i18nKey: string }> = [
  { regex: /Unknown config field "([^"]+)"/i, keys: ["name"], i18nKey: "error.unknownConfigField" },
  { regex: /Proxy name "([^"]+)" already in use/i, keys: ["name"], i18nKey: "error.proxyNameInUse" },
  { regex: /Port (\d+) already in use/i, keys: ["port"], i18nKey: "error.portInUse" },
  { regex: /Port (\d+) unavailable/i, keys: ["port"], i18nKey: "error.portUnavailable" },
  { regex: /Config parse error: (.+)/i, keys: ["detail"], i18nKey: "error.configParseError" },
  { regex: /TLS certificate verification failed: (.+)/i, keys: ["reason"], i18nKey: "error.tlsCertFailed" },
  { regex: /TLS error: (.+)/i, keys: ["detail"], i18nKey: "error.tlsError" },
  { regex: /Health check failed: (.+)/i, keys: ["detail"], i18nKey: "error.healthCheckFailed" },
];

const errorExact: Record<string, string> = {
  "Login to server failed": "error.loginFailed",
  "Token mismatch": "error.tokenMismatch",
  "Connection refused": "error.connectionRefused",
  "Connection reset": "error.connectionReset",
  "Network unreachable": "error.networkUnreachable",
  "Connection timeout": "error.connectionTimeout",
  "Reconnect timeout": "error.reconnectTimeout",
  "Control channel closed": "error.controlChannelClosed",
  "Permission denied": "error.permissionDenied",
  "Connection closed unexpectedly": "error.connectionClosedUnexpectedly",
};

export function translateError(error: string | null): string {
  if (!error) return "";
  const t = i18n.global.t;
  for (const { regex, keys, i18nKey } of errorPatterns) {
    const match = error.match(regex);
    if (match) {
      const args: Record<string, string> = {};
      keys.forEach((key, i) => { args[key] = match[i + 1]; });
      let result = t(i18nKey, args);
      Object.entries(args).forEach(([k, v]) => { result = result.replace(`{${k}}`, v); });
      return result;
    }
  }
  if (errorExact[error]) return t(errorExact[error]);
  return error;
}

export async function openLogFile(serverId: string): Promise<void> {
  return invoke("open_log_file", { serverId });
}

export interface AppUpdateInfo {
  current_version: string;
  latest_version: string;
  can_upgrade: boolean;
  download_url: string;
  install_method: string;
}

export async function checkAppUpdate(): Promise<AppUpdateInfo> {
  return invoke("check_app_update");
}

export async function downloadAppUpdate(version: string): Promise<void> {
  return invoke("download_app_update", { version });
}
