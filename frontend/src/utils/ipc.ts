import { invoke } from "@tauri-apps/api/core";
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
