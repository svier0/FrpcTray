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

const errorPatterns: Array<{ regex: RegExp; template: string }> = [
  { regex: /Unknown config field "([^"]+)"/i, template: 'Unknown config field "{name}"' },
  { regex: /Proxy name "([^"]+)" already in use/i, template: 'Proxy name "{name}" already in use' },
  { regex: /Port (\d+) already in use/i, template: "Port {port} already in use" },
  { regex: /Port (\d+) unavailable/i, template: "Port {port} unavailable" },
  { regex: /Config parse error: (.+)/i, template: "Config parse error: {detail}" },
  { regex: /TLS certificate verification failed: (.+)/i, template: "TLS certificate verification failed: {reason}" },
  { regex: /TLS error: (.+)/i, template: "TLS error: {detail}" },
  { regex: /Health check failed: (.+)/i, template: "Health check failed: {detail}" },
];

export function translateError(error: string | null): string {
  if (!error) return "";
  const { t, te } = i18n.global;
  for (const { regex, template } of errorPatterns) {
    const match = error.match(regex);
    if (match) {
      const keys: Record<string, string> = {};
      if (match[1]) {
        const placeholder = template.match(/\{(\w+)\}/)?.[1];
        if (placeholder) keys[placeholder] = match[1];
      }
      if (te(`error.${template}`)) {
        let result = t(`error.${template}`, keys);
        Object.entries(keys).forEach(([k, v]) => {
          result = result.replace(`{${k}}`, v);
        });
        return result;
      }
    }
  }
  if (te(`error.${error}`)) return t(`error.${error}`);
  return error;
}
