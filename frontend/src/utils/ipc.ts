import { invoke } from "@tauri-apps/api/core";
import type { ServerItem } from "../components/ServerItem.vue";

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
